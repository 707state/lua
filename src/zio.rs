use crate::lua_module::lua_State;
use core::ffi::{c_char, c_int, c_void};
use core::{ptr, slice};

pub const EOZ: c_int = -1;

pub type LuaReader =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void, *mut usize) -> *const c_char>;

#[repr(C)]
pub struct ZIO {
    pub n: usize,
    pub p: *const c_char,
    pub reader: LuaReader,
    pub data: *mut c_void,
    pub l: *mut lua_State,
}

#[repr(C)]
pub struct Mbuffer {
    pub buffer: *mut c_char,
    pub n: usize,
    pub buffsize: usize,
}

unsafe extern "C-unwind" {
    fn luaM_saferealloc_(
        state: *mut lua_State,
        block: *mut c_void,
        oldsize: usize,
        size: usize,
    ) -> *mut c_void;
}

impl ZIO {
    fn as_mut(z: *mut ZIO) -> &'static mut ZIO {
        unsafe { z.as_mut().expect("ZIO pointer must not be null") }
    }

    unsafe fn advance(&mut self, count: usize) {
        self.n -= count;
        self.p = unsafe { self.p.add(count) };
    }

    unsafe fn readable_bytes(&self, count: usize) -> &[u8] {
        unsafe { slice::from_raw_parts(self.p.cast::<u8>(), count) }
    }
}

impl Mbuffer {
    fn as_mut(buffer: *mut Mbuffer) -> &'static mut Mbuffer {
        unsafe { buffer.as_mut().expect("Mbuffer pointer must not be null") }
    }
}

fn checkbuffer(z: &mut ZIO) -> bool {
    if z.n == 0 {
        if unsafe { luaZ_fill(z) } == EOZ {
            return false;
        }
        z.n += 1;
        z.p = unsafe { z.p.sub(1) };
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaZ_fill(z: *mut ZIO) -> c_int {
    let z = ZIO::as_mut(z);
    let Some(reader) = z.reader else {
        return EOZ;
    };

    let mut size = 0usize;
    let buffer = unsafe { reader(z.l, z.data, &mut size) };
    if buffer.is_null() || size == 0 {
        return EOZ;
    }

    z.n = size - 1;
    z.p = unsafe { buffer.add(1) };
    unsafe { *buffer.cast::<u8>() as c_int }
}

pub(crate) unsafe fn luaZ_init(
    state: *mut lua_State,
    z: *mut ZIO,
    reader: LuaReader,
    data: *mut c_void,
) {
    let z = ZIO::as_mut(z);
    z.l = state;
    z.reader = reader;
    z.data = data;
    z.n = 0;
    z.p = ptr::null();
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaZ_initbuffer(_state: *mut lua_State, buffer: *mut Mbuffer) {
    let buffer = Mbuffer::as_mut(buffer);
    buffer.buffer = ptr::null_mut();
    buffer.n = 0;
    buffer.buffsize = 0;
}

pub(crate) unsafe fn luaZ_resizebuffer(
    state: *mut lua_State,
    buffer: *mut Mbuffer,
    size: usize,
) {
    let buffer = Mbuffer::as_mut(buffer);
    buffer.buffer =
        unsafe { luaM_saferealloc_(state, buffer.buffer.cast(), buffer.buffsize, size).cast() };
    buffer.buffsize = size;
    if buffer.n > size {
        buffer.n = size;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaZ_freebuffer(state: *mut lua_State, buffer: *mut Mbuffer) {
    unsafe { luaZ_resizebuffer(state, buffer, 0) };
}

pub(crate) unsafe fn luaZ_read(z: *mut ZIO, buffer: *mut c_void, mut n: usize) -> usize {
    let z = ZIO::as_mut(z);
    let mut out = buffer.cast::<u8>();

    while n != 0 {
        if !checkbuffer(z) {
            return n;
        }

        let chunk_len = z.n.min(n);
        let src = unsafe { z.readable_bytes(chunk_len) };
        let dst = unsafe { slice::from_raw_parts_mut(out, chunk_len) };
        dst.copy_from_slice(src);
        unsafe { z.advance(chunk_len) };
        out = unsafe { out.add(chunk_len) };
        n -= chunk_len;
    }

    0
}

pub(crate) unsafe fn luaZ_getaddr(z: *mut ZIO, n: usize) -> *const c_void {
    let z = ZIO::as_mut(z);
    if !checkbuffer(z) || z.n < n {
        return ptr::null();
    }

    let result = z.p.cast::<c_void>();
    unsafe { z.advance(n) };
    result
}

#[cfg(test)]
mod tests {
    use super::{
        Mbuffer, ZIO, luaZ_freebuffer, luaZ_getaddr, luaZ_init, luaZ_initbuffer, luaZ_read,
        luaZ_resizebuffer,
    };
    use crate::aux_rs::luaL_newstate;
    use crate::lua_module::lua_State;
    use crate::luaffi::lua_close;
    use core::ffi::{c_char, c_void};
    use core::{ptr, slice};

    struct ReaderData {
        chunks: &'static [&'static [u8]],
        index: usize,
    }

    unsafe extern "C-unwind" fn reader(
        _state: *mut lua_State,
        data: *mut c_void,
        size: *mut usize,
    ) -> *const c_char {
        let data = unsafe { &mut *data.cast::<ReaderData>() };
        if let Some(chunk) = data.chunks.get(data.index) {
            data.index += 1;
            unsafe { *size = chunk.len() };
            chunk.as_ptr().cast()
        } else {
            unsafe { *size = 0 };
            ptr::null()
        }
    }

    #[test]
    fn read_spans_reader_chunks() {
        let mut data = ReaderData {
            chunks: &[b"ab", b"c", b"de"],
            index: 0,
        };
        let mut z = ZIO {
            n: 0,
            p: ptr::null(),
            reader: None,
            data: ptr::null_mut(),
            l: ptr::null_mut(),
        };

        unsafe {
            luaZ_init(
                ptr::null_mut(),
                &mut z,
                Some(reader),
                (&mut data as *mut ReaderData).cast(),
            )
        };

        let mut out = [0u8; 5];
        let missing = unsafe { luaZ_read(&mut z, out.as_mut_ptr().cast(), out.len()) };
        assert_eq!(missing, 0);
        assert_eq!(&out, b"abcde");

        let mut extra = [0u8; 1];
        let missing = unsafe { luaZ_read(&mut z, extra.as_mut_ptr().cast(), extra.len()) };
        assert_eq!(missing, 1);
    }

    #[test]
    fn getaddr_only_returns_contiguous_blocks() {
        let mut data = ReaderData {
            chunks: &[b"ab", b"cde"],
            index: 0,
        };
        let mut z = ZIO {
            n: 0,
            p: ptr::null(),
            reader: None,
            data: ptr::null_mut(),
            l: ptr::null_mut(),
        };

        unsafe {
            luaZ_init(
                ptr::null_mut(),
                &mut z,
                Some(reader),
                (&mut data as *mut ReaderData).cast(),
            )
        };

        let first = unsafe { luaZ_getaddr(&mut z, 2) };
        assert_eq!(
            unsafe { slice::from_raw_parts(first.cast::<u8>(), 2) },
            b"ab"
        );

        let second = unsafe { luaZ_getaddr(&mut z, 2) };
        assert_eq!(
            unsafe { slice::from_raw_parts(second.cast::<u8>(), 2) },
            b"cd"
        );

        let third = unsafe { luaZ_getaddr(&mut z, 2) };
        assert!(third.is_null());

        let mut last = [0u8; 1];
        let missing = unsafe { luaZ_read(&mut z, last.as_mut_ptr().cast(), last.len()) };
        assert_eq!(missing, 0);
        assert_eq!(&last, b"e");

        let missing = unsafe { luaZ_read(&mut z, last.as_mut_ptr().cast(), last.len()) };
        assert_eq!(missing, 1);
    }

    #[test]
    fn mbuffer_resize_and_free_round_trip() {
        let state = luaL_newstate();
        assert!(!state.is_null());

        let mut buffer = Mbuffer {
            buffer: ptr::null_mut(),
            n: usize::MAX,
            buffsize: usize::MAX,
        };

        unsafe { luaZ_initbuffer(state, &mut buffer) };
        assert!(buffer.buffer.is_null());
        assert_eq!(buffer.n, 0);
        assert_eq!(buffer.buffsize, 0);

        unsafe { luaZ_resizebuffer(state, &mut buffer, 8) };
        assert!(!buffer.buffer.is_null());
        assert_eq!(buffer.buffsize, 8);

        unsafe {
            let slice = slice::from_raw_parts_mut(buffer.buffer.cast::<u8>(), 8);
            slice.copy_from_slice(b"12345678");
        }

        buffer.n = 8;
        unsafe { luaZ_resizebuffer(state, &mut buffer, 4) };
        assert_eq!(buffer.buffsize, 4);
        assert_eq!(buffer.n, 4);
        assert_eq!(
            unsafe { slice::from_raw_parts(buffer.buffer.cast::<u8>(), 4) },
            b"1234"
        );

        unsafe { luaZ_freebuffer(state, &mut buffer) };
        assert!(buffer.buffer.is_null());
        assert_eq!(buffer.buffsize, 0);

        unsafe { lua_close(state) };
    }
}
