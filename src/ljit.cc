/*
** LLVM ORC JIT support for the Lua VM
** See Copyright Notice in lua.h
*/

#include <memory>
#include <mutex>
#include <string>
#include <unordered_map>
#include <vector>

#include "llvm/ExecutionEngine/Orc/AbsoluteSymbols.h"
#include "llvm/ExecutionEngine/Orc/LLJIT.h"
#include "llvm/ExecutionEngine/Orc/Mangling.h"
#include "llvm/ExecutionEngine/Orc/ThreadSafeModule.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/Support/Error.h"
#include "llvm/Support/TargetSelect.h"

#define ljit_c
#define LUA_CORE

#include "lua.h"

#include "ldo.h"
#include "lfunc.h"
#include "lgc.h"
#include "ljit.h"
#include "llimits.h"
#include "lobject.h"
#include "lopcodes.h"
#include "lstate.h"

typedef int (*LuaJITFunction)(lua_State *L);

enum {
  LUAJIT_ARITH_ADD,
  LUAJIT_ARITH_SUB,
  LUAJIT_ARITH_MUL,
  LUAJIT_ARITH_DIV,
  LUAJIT_ARITH_MOD,
  LUAJIT_ARITH_POW,
  LUAJIT_ARITH_UNM
};

enum {
  LUAJIT_CMP_EQ,
  LUAJIT_CMP_LT,
  LUAJIT_CMP_LE
};

extern "C" {

static StkId luaJIT_base(lua_State *L) {
  return L->base;
}

static TValue *luaJIT_constant(lua_State *L, int index) {
  LClosure *cl = &clvalue(L->ci->func)->l;
  return &cl->p->k[index];
}

static TValue *luaJIT_reg(lua_State *L, int index) {
  return L->base + index;
}

static void luaJIT_copy(lua_State *L, int dst, int src) {
  StkId base = L->base;
  setobjs2s(L, base + dst, base + src);
}

static void luaJIT_loadk(lua_State *L, int dst, int kidx) {
  LClosure *cl = &clvalue(L->ci->func)->l;
  setobj2s(L, L->base + dst, &cl->p->k[kidx]);
}

static void luaJIT_loadbool(lua_State *L, int dst, int value) {
  setbvalue(L->base + dst, value);
}

static void luaJIT_loadnil(lua_State *L, int from, int to) {
  StkId base = L->base;
  for (int i = from; i <= to; i++)
    setnilvalue(base + i);
}

static void luaJIT_setnum(lua_State *L, int dst, lua_Number value) {
  setnvalue(L->base + dst, value);
}

static int luaJIT_isnum(lua_State *L, int reg) {
  return ttisnumber(L->base + reg);
}

static int luaJIT_arith(lua_State *L, int op, int dst, int left, int right) {
  StkId base = L->base;
  TValue *rb = base + left;
  TValue *rc = base + right;
  if (!ttisnumber(rb) || !ttisnumber(rc))
    return 0;
  lua_Number nb = nvalue(rb);
  lua_Number nc = nvalue(rc);
  switch (op) {
    case LUAJIT_ARITH_ADD: setnvalue(base + dst, luai_numadd(nb, nc)); return 1;
    case LUAJIT_ARITH_SUB: setnvalue(base + dst, luai_numsub(nb, nc)); return 1;
    case LUAJIT_ARITH_MUL: setnvalue(base + dst, luai_nummul(nb, nc)); return 1;
    case LUAJIT_ARITH_DIV: setnvalue(base + dst, luai_numdiv(nb, nc)); return 1;
    case LUAJIT_ARITH_MOD: setnvalue(base + dst, luai_nummod(nb, nc)); return 1;
    case LUAJIT_ARITH_POW: setnvalue(base + dst, luai_numpow(nb, nc)); return 1;
    default: return 0;
  }
}

static int luaJIT_arithrk(lua_State *L, int op, int dst, int left,
                          int leftK, int right, int rightK) {
  LClosure *cl = &clvalue(L->ci->func)->l;
  TValue *ra = L->base + dst;
  const TValue *rb = leftK ? &cl->p->k[left] : L->base + left;
  const TValue *rc = rightK ? &cl->p->k[right] : L->base + right;
  if (!ttisnumber(rb) || !ttisnumber(rc))
    return 0;
  lua_Number nb = nvalue(rb);
  lua_Number nc = nvalue(rc);
  switch (op) {
    case LUAJIT_ARITH_ADD: setnvalue(ra, luai_numadd(nb, nc)); return 1;
    case LUAJIT_ARITH_SUB: setnvalue(ra, luai_numsub(nb, nc)); return 1;
    case LUAJIT_ARITH_MUL: setnvalue(ra, luai_nummul(nb, nc)); return 1;
    case LUAJIT_ARITH_DIV: setnvalue(ra, luai_numdiv(nb, nc)); return 1;
    case LUAJIT_ARITH_MOD: setnvalue(ra, luai_nummod(nb, nc)); return 1;
    case LUAJIT_ARITH_POW: setnvalue(ra, luai_numpow(nb, nc)); return 1;
    default: return 0;
  }
}

static int luaJIT_unm(lua_State *L, int dst, int src) {
  StkId base = L->base;
  TValue *rb = base + src;
  if (!ttisnumber(rb))
    return 0;
  setnvalue(base + dst, luai_numunm(nvalue(rb)));
  return 1;
}

static void luaJIT_not(lua_State *L, int dst, int src) {
  StkId base = L->base;
  setbvalue(base + dst, l_isfalse(base + src));
}

static int luaJIT_cmp(lua_State *L, int op, int left, int right) {
  StkId base = L->base;
  TValue *rb = base + left;
  TValue *rc = base + right;
  if (!ttisnumber(rb) || !ttisnumber(rc))
    return 0;
  switch (op) {
    case LUAJIT_CMP_EQ: return luai_numeq(nvalue(rb), nvalue(rc));
    case LUAJIT_CMP_LT: return luai_numlt(nvalue(rb), nvalue(rc));
    case LUAJIT_CMP_LE: return luai_numle(nvalue(rb), nvalue(rc));
    default: return 0;
  }
}

static int luaJIT_cmprk(lua_State *L, int op, int left, int leftK,
                        int right, int rightK) {
  LClosure *cl = &clvalue(L->ci->func)->l;
  const TValue *rb = leftK ? &cl->p->k[left] : L->base + left;
  const TValue *rc = rightK ? &cl->p->k[right] : L->base + right;
  if (!ttisnumber(rb) || !ttisnumber(rc))
    return 0;
  switch (op) {
    case LUAJIT_CMP_EQ: return luai_numeq(nvalue(rb), nvalue(rc));
    case LUAJIT_CMP_LT: return luai_numlt(nvalue(rb), nvalue(rc));
    case LUAJIT_CMP_LE: return luai_numle(nvalue(rb), nvalue(rc));
    default: return 0;
  }
}

static int luaJIT_forprep(lua_State *L, int a) {
  StkId ra = L->base + a;
  if (!ttisnumber(ra) || !ttisnumber(ra + 1) || !ttisnumber(ra + 2))
    return 0;
  setnvalue(ra, luai_numsub(nvalue(ra), nvalue(ra + 2)));
  return 1;
}

static int luaJIT_forloop(lua_State *L, int a) {
  StkId ra = L->base + a;
  lua_Number step = nvalue(ra + 2);
  lua_Number idx = luai_numadd(nvalue(ra), step);
  lua_Number limit = nvalue(ra + 1);
  if (luai_numlt(0, step) ? luai_numle(idx, limit) : luai_numle(limit, idx)) {
    setnvalue(ra, idx);
    setnvalue(ra + 3, idx);
    return 1;
  }
  return 0;
}

static int luaJIT_finish_return(lua_State *L, int a, int b, int pc) {
  StkId ra = L->base + a;
  if (b == 0)
    return 0;
  L->top = ra + b - 1;
  if (L->openupval)
    luaF_close(L, L->base);
  LClosure *cl = &clvalue(L->ci->func)->l;
  L->savedpc = cl->p->code + pc;
  luaD_poscall(L, ra);
  return 1;
}

}

namespace {

struct CompiledProto {
  LuaJITFunction Fn;
  bool Supported;
};

class OrcLuaJIT {
public:
  static OrcLuaJIT &instance() {
    static OrcLuaJIT J;
    return J;
  }

  LuaJITFunction getFunction(Proto *P) {
    std::lock_guard<std::mutex> Lock(Mutex);
    auto It = Cache.find(P);
    if (It != Cache.end())
      return It->second.Supported ? It->second.Fn : nullptr;

    if (!isSupported(P)) {
      Cache[P] = {nullptr, false};
      return nullptr;
    }

    LuaJITFunction Fn = compile(P);
    Cache[P] = {Fn, Fn != nullptr};
    return Fn;
  }

private:
  OrcLuaJIT() {
    llvm::InitializeNativeTarget();
    llvm::InitializeNativeTargetAsmPrinter();
    auto J = llvm::orc::LLJITBuilder().create();
    if (!J) {
      llvm::consumeError(J.takeError());
      return;
    }
    JIT = std::move(*J);
    defineRuntimeSymbols();
  }

  void defineRuntimeSymbols() {
    if (!JIT)
      return;
    llvm::orc::MangleAndInterner Mangle(JIT->getExecutionSession(),
                                        JIT->getDataLayout());
    llvm::orc::SymbolMap Symbols;
    auto Add = [&](const char *Name, void *Ptr) {
      Symbols[Mangle(Name)] = llvm::orc::ExecutorSymbolDef(
          llvm::orc::ExecutorAddr::fromPtr(Ptr),
          llvm::JITSymbolFlags::Exported);
    };
    Add("luaJIT_base", reinterpret_cast<void *>(&luaJIT_base));
    Add("luaJIT_constant", reinterpret_cast<void *>(&luaJIT_constant));
    Add("luaJIT_reg", reinterpret_cast<void *>(&luaJIT_reg));
    Add("luaJIT_copy", reinterpret_cast<void *>(&luaJIT_copy));
    Add("luaJIT_loadk", reinterpret_cast<void *>(&luaJIT_loadk));
    Add("luaJIT_loadbool", reinterpret_cast<void *>(&luaJIT_loadbool));
    Add("luaJIT_loadnil", reinterpret_cast<void *>(&luaJIT_loadnil));
    Add("luaJIT_setnum", reinterpret_cast<void *>(&luaJIT_setnum));
    Add("luaJIT_isnum", reinterpret_cast<void *>(&luaJIT_isnum));
    Add("luaJIT_arith", reinterpret_cast<void *>(&luaJIT_arith));
    Add("luaJIT_arithrk", reinterpret_cast<void *>(&luaJIT_arithrk));
    Add("luaJIT_unm", reinterpret_cast<void *>(&luaJIT_unm));
    Add("luaJIT_not", reinterpret_cast<void *>(&luaJIT_not));
    Add("luaJIT_cmp", reinterpret_cast<void *>(&luaJIT_cmp));
    Add("luaJIT_cmprk", reinterpret_cast<void *>(&luaJIT_cmprk));
    Add("luaJIT_forprep", reinterpret_cast<void *>(&luaJIT_forprep));
    Add("luaJIT_forloop", reinterpret_cast<void *>(&luaJIT_forloop));
    Add("luaJIT_finish_return", reinterpret_cast<void *>(&luaJIT_finish_return));
    if (auto Err = JIT->getMainJITDylib().define(
            llvm::orc::absoluteSymbols(std::move(Symbols)))) {
      llvm::consumeError(std::move(Err));
      JIT.reset();
    }
  }

  bool isNumberConstant(Proto *P, int K) const {
    return K >= 0 && K < P->sizek && ttisnumber(&P->k[K]);
  }

  bool isSupportedRK(Proto *P, int RK) const {
    return !ISK(RK) || isNumberConstant(P, INDEXK(RK));
  }

  bool isSupported(Proto *P) const {
    if (!JIT || P->numparams != 0 || P->is_vararg || P->nups != 0 ||
        P->sizep != 0 || P->maxstacksize == 0)
      return false;

    enum RegType { Unknown, Number, Boolean, Nil };
    std::vector<RegType> Types(P->maxstacksize, Unknown);
    auto RkIsNumber = [&](int RK) {
      if (ISK(RK))
        return isNumberConstant(P, INDEXK(RK));
      return RK >= 0 && RK < P->maxstacksize && Types[RK] == Number;
    };

    for (int pc = 0; pc < P->sizecode; pc++) {
      Instruction I = P->code[pc];
      switch (GET_OPCODE(I)) {
        case OP_MOVE:
          Types[GETARG_A(I)] = Types[GETARG_B(I)];
          break;
        case OP_LOADBOOL:
          Types[GETARG_A(I)] = Boolean;
          break;
        case OP_LOADNIL:
          for (int i = GETARG_A(I); i <= GETARG_B(I); i++)
            Types[i] = Nil;
          break;
        case OP_JMP:
          break;
        case OP_NOT:
          Types[GETARG_A(I)] = Boolean;
          break;
        case OP_LOADK:
          if (!isNumberConstant(P, GETARG_Bx(I)))
            return false;
          Types[GETARG_A(I)] = Number;
          break;
        case OP_ADD:
        case OP_SUB:
        case OP_MUL:
        case OP_DIV:
        case OP_MOD:
        case OP_POW:
          if (!RkIsNumber(GETARG_B(I)) || !RkIsNumber(GETARG_C(I)))
            return false;
          Types[GETARG_A(I)] = Number;
          break;
        case OP_EQ:
        case OP_LT:
        case OP_LE:
          if (!RkIsNumber(GETARG_B(I)) || !RkIsNumber(GETARG_C(I)))
            return false;
          break;
        case OP_UNM:
          if (GETARG_B(I) < 0 || GETARG_B(I) >= P->maxstacksize ||
              Types[GETARG_B(I)] != Number)
            return false;
          Types[GETARG_A(I)] = Number;
          break;
        case OP_FORPREP:
          if (GETARG_A(I) + 2 >= P->maxstacksize ||
              Types[GETARG_A(I)] != Number ||
              Types[GETARG_A(I) + 1] != Number ||
              Types[GETARG_A(I) + 2] != Number)
            return false;
          break;
        case OP_FORLOOP:
          if (GETARG_A(I) + 3 >= P->maxstacksize)
            return false;
          Types[GETARG_A(I)] = Number;
          Types[GETARG_A(I) + 3] = Number;
          break;
        case OP_RETURN:
          if (GETARG_B(I) == 0)
            return false;
          break;
        default:
          return false;
      }
    }
    return P->sizecode > 0 &&
           GET_OPCODE(P->code[P->sizecode - 1]) == OP_RETURN;
  }

  LuaJITFunction compile(Proto *P) {
    auto Ctx = std::make_unique<llvm::LLVMContext>();
    auto M = std::make_unique<llvm::Module>("lua.orcjit", *Ctx);
    M->setDataLayout(JIT->getDataLayout());
    llvm::IRBuilder<> B(*Ctx);

    llvm::Type *VoidTy = B.getVoidTy();
    llvm::Type *I1Ty = B.getInt1Ty();
    llvm::Type *I32Ty = B.getInt32Ty();
    llvm::PointerType *PtrTy = B.getPtrTy();

    auto FnTy = llvm::FunctionType::get(I32Ty, {PtrTy}, false);
    std::string Name = "lua_jit_proto_" +
                       std::to_string(reinterpret_cast<uintptr_t>(P));
    llvm::Function *Fn = llvm::Function::Create(
        FnTy, llvm::Function::ExternalLinkage, Name, M.get());
    llvm::Value *L = Fn->getArg(0);

    auto HelperVoid2 = llvm::FunctionType::get(VoidTy, {PtrTy, I32Ty, I32Ty}, false);
    auto HelperI32I32 = llvm::FunctionType::get(I32Ty, {PtrTy, I32Ty}, false);
    auto HelperArith = llvm::FunctionType::get(I32Ty, {PtrTy, I32Ty, I32Ty, I32Ty, I32Ty}, false);
    auto HelperArithRK = llvm::FunctionType::get(
        I32Ty, {PtrTy, I32Ty, I32Ty, I32Ty, I32Ty, I32Ty, I32Ty}, false);
    auto HelperReturn = llvm::FunctionType::get(I32Ty, {PtrTy, I32Ty, I32Ty, I32Ty}, false);
    auto CopyF = M->getOrInsertFunction("luaJIT_copy", HelperVoid2);
    auto LoadKF = M->getOrInsertFunction("luaJIT_loadk", HelperVoid2);
    auto LoadBoolF = M->getOrInsertFunction("luaJIT_loadbool", HelperVoid2);
    auto LoadNilF = M->getOrInsertFunction("luaJIT_loadnil", HelperVoid2);
    auto ArithF = M->getOrInsertFunction("luaJIT_arith", HelperArith);
    auto ArithRKF = M->getOrInsertFunction("luaJIT_arithrk", HelperArithRK);
    auto UnmF = M->getOrInsertFunction("luaJIT_unm", llvm::FunctionType::get(I32Ty, {PtrTy, I32Ty, I32Ty}, false));
    auto NotF = M->getOrInsertFunction("luaJIT_not", HelperVoid2);
    auto CmpF = M->getOrInsertFunction("luaJIT_cmp", llvm::FunctionType::get(I32Ty, {PtrTy, I32Ty, I32Ty, I32Ty}, false));
    auto CmpRKF = M->getOrInsertFunction("luaJIT_cmprk", llvm::FunctionType::get(I32Ty, {PtrTy, I32Ty, I32Ty, I32Ty, I32Ty, I32Ty}, false));
    auto ForprepF = M->getOrInsertFunction("luaJIT_forprep", HelperI32I32);
    auto ForloopF = M->getOrInsertFunction("luaJIT_forloop", HelperI32I32);
    auto ReturnF = M->getOrInsertFunction("luaJIT_finish_return", HelperReturn);

    std::vector<llvm::BasicBlock *> Blocks;
    Blocks.reserve(P->sizecode + 1);
    for (int pc = 0; pc < P->sizecode; pc++)
      Blocks.push_back(llvm::BasicBlock::Create(*Ctx, "pc" + std::to_string(pc), Fn));
    llvm::BasicBlock *Fallback = llvm::BasicBlock::Create(*Ctx, "fallback", Fn);

    B.SetInsertPoint(llvm::BasicBlock::Create(*Ctx, "entry", Fn));
    B.CreateBr(Blocks[0]);

    auto C = [&](int V) { return llvm::ConstantInt::get(I32Ty, V); };
    auto BranchNext = [&](int PC) {
      if (PC + 1 < P->sizecode)
        B.CreateBr(Blocks[PC + 1]);
      else
        B.CreateRet(C(0));
    };
    auto EmitGuarded = [&](llvm::CallInst *Call, int NextPC) {
      llvm::Value *Ok = B.CreateICmpNE(Call, C(0));
      llvm::BasicBlock *Next = NextPC < P->sizecode ? Blocks[NextPC] : Fallback;
      B.CreateCondBr(Ok, Next, Fallback);
    };
    for (int pc = 0; pc < P->sizecode; pc++) {
      B.SetInsertPoint(Blocks[pc]);
      Instruction I = P->code[pc];
      switch (GET_OPCODE(I)) {
        case OP_MOVE:
          B.CreateCall(CopyF, {L, C(GETARG_A(I)), C(GETARG_B(I))});
          BranchNext(pc);
          break;
        case OP_LOADK:
          B.CreateCall(LoadKF, {L, C(GETARG_A(I)), C(GETARG_Bx(I))});
          BranchNext(pc);
          break;
        case OP_LOADBOOL:
          B.CreateCall(LoadBoolF, {L, C(GETARG_A(I)), C(GETARG_B(I))});
          if (GETARG_C(I))
            B.CreateBr(pc + 2 < P->sizecode ? Blocks[pc + 2] : Fallback);
          else
            BranchNext(pc);
          break;
        case OP_LOADNIL:
          B.CreateCall(LoadNilF, {L, C(GETARG_A(I)), C(GETARG_B(I))});
          BranchNext(pc);
          break;
        case OP_ADD:
        case OP_SUB:
        case OP_MUL:
        case OP_DIV:
        case OP_MOD:
        case OP_POW: {
          int Op = GET_OPCODE(I) - OP_ADD;
          int BRK = GETARG_B(I);
          int CRK = GETARG_C(I);
          auto *Call = B.CreateCall(
              ArithRKF, {L, C(Op), C(GETARG_A(I)),
                         C(ISK(BRK) ? INDEXK(BRK) : BRK), C(ISK(BRK)),
                         C(ISK(CRK) ? INDEXK(CRK) : CRK), C(ISK(CRK))});
          EmitGuarded(Call, pc + 1);
          break;
        }
        case OP_UNM: {
          auto *Call = B.CreateCall(UnmF, {L, C(GETARG_A(I)), C(GETARG_B(I))});
          EmitGuarded(Call, pc + 1);
          break;
        }
        case OP_NOT:
          B.CreateCall(NotF, {L, C(GETARG_A(I)), C(GETARG_B(I))});
          BranchNext(pc);
          break;
        case OP_JMP:
          B.CreateBr(Blocks[pc + 1 + GETARG_sBx(I)]);
          break;
        case OP_EQ:
        case OP_LT:
        case OP_LE: {
          int Op = GET_OPCODE(I) == OP_EQ ? LUAJIT_CMP_EQ :
                   GET_OPCODE(I) == OP_LT ? LUAJIT_CMP_LT : LUAJIT_CMP_LE;
          int BRK = GETARG_B(I);
          int CRK = GETARG_C(I);
          llvm::Value *Result = B.CreateCall(
              CmpRKF, {L, C(Op),
                       C(ISK(BRK) ? INDEXK(BRK) : BRK), C(ISK(BRK)),
                       C(ISK(CRK) ? INDEXK(CRK) : CRK), C(ISK(CRK))});
          llvm::Value *Matches = B.CreateICmpEQ(Result, C(GETARG_A(I)));
          int Target = pc + 2 + GETARG_sBx(P->code[pc + 1]);
          B.CreateCondBr(Matches, Blocks[Target], Blocks[pc + 2]);
          break;
        }
        case OP_FORPREP: {
          auto *Call = B.CreateCall(ForprepF, {L, C(GETARG_A(I))});
          llvm::Value *Ok = B.CreateICmpNE(Call, C(0));
          int Target = pc + 1 + GETARG_sBx(I);
          B.CreateCondBr(Ok, Blocks[Target], Fallback);
          break;
        }
        case OP_FORLOOP: {
          llvm::Value *Again = B.CreateICmpNE(
              B.CreateCall(ForloopF, {L, C(GETARG_A(I))}), C(0));
          int Target = pc + 1 + GETARG_sBx(I);
          B.CreateCondBr(Again, Blocks[Target],
                         pc + 1 < P->sizecode ? Blocks[pc + 1] : Fallback);
          break;
        }
        case OP_RETURN:
          B.CreateRet(B.CreateCall(ReturnF, {L, C(GETARG_A(I)), C(GETARG_B(I)), C(pc + 1)}));
          break;
        default:
          B.CreateRet(C(0));
          break;
      }
    }

    B.SetInsertPoint(Fallback);
    B.CreateRet(C(0));

    if (auto Err = JIT->addIRModule(
            llvm::orc::ThreadSafeModule(std::move(M), std::move(Ctx)))) {
      llvm::consumeError(std::move(Err));
      return nullptr;
    }

    auto Sym = JIT->lookup(Name);
    if (!Sym) {
      llvm::consumeError(Sym.takeError());
      return nullptr;
    }
    return Sym->toPtr<LuaJITFunction>();
  }

  std::mutex Mutex;
  std::unique_ptr<llvm::orc::LLJIT> JIT;
  std::unordered_map<Proto *, CompiledProto> Cache;
};

}  /* namespace */

int luaJIT_execute(lua_State *L) {
  if (L->hookmask != 0 || !isLua(L->ci))
    return 0;
  LClosure *cl = &clvalue(L->ci->func)->l;
  if (L->savedpc != cl->p->code)
    return 0;
  LuaJITFunction Fn = OrcLuaJIT::instance().getFunction(cl->p);
  if (Fn == nullptr)
    return 0;
  return Fn(L);
}
