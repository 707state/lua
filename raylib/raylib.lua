--[[
  Copyright (C) 2020 Astie Teddy

  Permission to use, copy, modify, and/or distribute this software for any
  purpose with or without fee is hereby granted, provided that the above
  copyright notice and this permission notice appear in all copies.

  THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
  WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
  MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
  ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
  WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
  OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
  CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
--]]
local ffi=require('ffi')
local C = ffi.C

local rl = {}
setmetatable(rl, rl)

-- structs definition
ffi.cdef [[
  typedef struct Vector2 {
    float x;
    float y;
  } Vector2;

  typedef struct Vector3 {
    float x;
    float y;
    float z;
  } Vector3;

  typedef struct Vector4 {
    float x;
    float y;
    float z;
    float w;
  } Vector4;

  typedef Vector4 Quaternion;

  typedef struct Matrix {
    float m0, m4, m8, m12;
    float m1, m5, m9, m13;
    float m2, m6, m10, m14;
    float m3, m7, m11, m15;
  } Matrix;

  typedef struct Color {
    unsigned char r;
    unsigned char g;
    unsigned char b;
    unsigned char a;
  } Color;

  typedef struct Rectangle {
    float x;
    float y;
    float width;
    float height;
  } Rectangle;

  typedef struct Image {
    void *data;
    int width;
    int height;
    int mipmaps;
    int format;
  } Image;

  typedef struct Texture {
    unsigned int id;
    int width;
    int height;
    int mipmaps;
    int format;
  } Texture;
  typedef Texture Texture2D;
  typedef Texture TextureCubemap;

  typedef struct RenderTexture {
    unsigned int id;
    Texture texture;
    Texture depth;
  } RenderTexture;
  typedef RenderTexture RenderTexture2D;

  typedef struct NPatchInfo {
    Rectangle sourceRec;
    int left;
    int top;
    int right;
    int bottom;
    int layout;
  } NPatchInfo;

  typedef struct GlyphInfo {
    int value;
    int offsetX;
    int offsetY;
    int advanceX;
    Image image;
  } GlyphInfo;

  typedef struct Font {
    int baseSize;
    int glyphCount;
    int glyphPadding;
    Texture2D texture;
    Rectangle *recs;
    GlyphInfo *glyphs;
  } Font;

  typedef Font SpriteFont;
  typedef struct Camera3D {
    Vector3 position;
    Vector3 target;
    Vector3 up;
    float fovy;
    int projection;
  } Camera3D;

  typedef Camera3D Camera;
  typedef struct Camera2D {
    Vector2 offset;
    Vector2 target;
    float rotation;
    float zoom;
  } Camera2D;

  typedef struct Mesh {
    int vertexCount;
    int triangleCount;
    float *vertices;
    float *texcoords;
    float *texcoords2;
    float *normals;
    float *tangents;
    unsigned char *colors;
    unsigned short *indices;
    float *animVertices;
    float *animNormals;
    int *boneIds;
    float *boneWeights;
    unsigned int vaoId;
    unsigned int *vboId;
  } Mesh;

  typedef struct Shader {
    unsigned int id;
    int *locs;
  } Shader;

  typedef struct MaterialMap {
    Texture2D texture;
    Color color;
    float value;
  } MaterialMap;

  typedef struct Material {
    Shader shader;
    MaterialMap *maps;
    float params[4];
  } Material;

  typedef struct Transform {
    Vector3 translation;
    Quaternion rotation;
    Vector3 scale;
  } Transform;

  typedef struct BoneInfo {
    char name[32];
    int parent;
  } BoneInfo;

  typedef struct Model {
    Matrix transform;

    int meshCount;

    int materialCount;
    Mesh *meshes;
    Material *materials;
    int *meshMaterial;
    int boneCount;
    BoneInfo *bones;
    Transform *bindPose;
  } Model;

  typedef struct ModelAnimation {
    int boneCount;
    int frameCount;
    BoneInfo *bones;
    Transform **framePoses;
    char name[32];
  } ModelAnimation;

  typedef struct Ray {
    Vector3 position;
    Vector3 direction;
  } Ray;

  typedef struct RayCollision {
    bool hit;
    float distance;
    Vector3 point;
    Vector3 normal;
  } RayCollision;

  typedef struct BoundingBox {
    Vector3 min;
    Vector3 max;
  } BoundingBox;

  typedef struct Wave {
    unsigned int frameCount;
    unsigned int sampleRate;
    unsigned int sampleSize;
    unsigned int channels;
    void *data;
  } Wave;

  typedef struct rAudioBuffer rAudioBuffer;
  typedef struct rAudioProcessor rAudioProcessor;

  typedef struct AudioStream {
    rAudioBuffer *buffer;
    rAudioProcessor *processor;

    unsigned int sampleRate;
    unsigned int sampleSize;
    unsigned int channels;
  } AudioStream;

  typedef struct Sound {
    AudioStream stream;
    unsigned int frameCount;
  } Sound;

  typedef struct Music {
    AudioStream stream;
    unsigned int frameCount;
    bool looping;

    int ctxType;
    void *ctxData;
  } Music;

  typedef struct VrDeviceInfo {
    int hResolution;
    int vResolution;
    float hScreenSize;
    float vScreenSize;
    float vScreenCenter;
    float eyeToScreenDistance;
    float lensSeparationDistance;
    float interpupillaryDistance;
    float lensDistortionValues[4];
    float chromaAbCorrection[4];
  } VrDeviceInfo;

  typedef struct VrStereoConfig {
    Matrix projection[2];
    Matrix viewOffset[2];
    float leftLensCenter[2];
    float rightLensCenter[2];
    float leftScreenCenter[2];
    float rightScreenCenter[2];
    float scale[2];
    float scaleIn[2];
  } VrStereoConfig;

  typedef struct FilePathList {
    unsigned int capacity;
    unsigned int count;
    char **paths;
  } FilePathList;

  typedef struct AutomationEvent {
    unsigned int frame;
    unsigned int type;
    int params[4];
  } AutomationEvent;

  typedef struct AutomationEventList {
    unsigned int capacity;
    unsigned int count;
    AutomationEvent *events;
  } AutomationEventList;

  typedef enum {
    FLAG_VSYNC_HINT = 0x00000040,
    FLAG_FULLSCREEN_MODE = 0x00000002,
    FLAG_WINDOW_RESIZABLE = 0x00000004,
    FLAG_WINDOW_UNDECORATED = 0x00000008,
    FLAG_WINDOW_HIDDEN = 0x00000080,
    FLAG_WINDOW_MINIMIZED = 0x00000200,
    FLAG_WINDOW_MAXIMIZED = 0x00000400,
    FLAG_WINDOW_UNFOCUSED = 0x00000800,
    FLAG_WINDOW_TOPMOST = 0x00001000,
    FLAG_WINDOW_ALWAYS_RUN = 0x00000100,
    FLAG_WINDOW_TRANSPARENT = 0x00000010,
    FLAG_WINDOW_HIGHDPI = 0x00002000,
    FLAG_WINDOW_MOUSE_PASSTHROUGH = 0x00004000,
    FLAG_BORDERLESS_WINDOWED_MODE = 0x00008000,
    FLAG_MSAA_4X_HINT = 0x00000020,
    FLAG_INTERLACED_HINT = 0x00010000
  } ConfigFlags;

  typedef enum {
    LOG_ALL = 0,
    LOG_TRACE,
    LOG_DEBUG,
    LOG_INFO,
    LOG_WARNING,
    LOG_ERROR,
    LOG_FATAL,
    LOG_NONE
  } TraceLogLevel;

  typedef enum {
    KEY_NULL = 0,
    KEY_APOSTROPHE = 39,
    KEY_COMMA = 44,
    KEY_MINUS = 45,
    KEY_PERIOD = 46,
    KEY_SLASH = 47,
    KEY_ZERO = 48,
    KEY_ONE = 49,
    KEY_TWO = 50,
    KEY_THREE = 51,
    KEY_FOUR = 52,
    KEY_FIVE = 53,
    KEY_SIX = 54,
    KEY_SEVEN = 55,
    KEY_EIGHT = 56,
    KEY_NINE = 57,
    KEY_SEMICOLON = 59,
    KEY_EQUAL = 61,
    KEY_A = 65,
    KEY_B = 66,
    KEY_C = 67,
    KEY_D = 68,
    KEY_E = 69,
    KEY_F = 70,
    KEY_G = 71,
    KEY_H = 72,
    KEY_I = 73,
    KEY_J = 74,
    KEY_K = 75,
    KEY_L = 76,
    KEY_M = 77,
    KEY_N = 78,
    KEY_O = 79,
    KEY_P = 80,
    KEY_Q = 81,
    KEY_R = 82,
    KEY_S = 83,
    KEY_T = 84,
    KEY_U = 85,
    KEY_V = 86,
    KEY_W = 87,
    KEY_X = 88,
    KEY_Y = 89,
    KEY_Z = 90,
    KEY_LEFT_BRACKET = 91,
    KEY_BACKSLASH = 92,
    KEY_RIGHT_BRACKET = 93,
    KEY_GRAVE = 96,
    KEY_SPACE = 32,
    KEY_ESCAPE = 256,
    KEY_ENTER = 257,
    KEY_TAB = 258,
    KEY_BACKSPACE = 259,
    KEY_INSERT= 260,
    KEY_DELETE= 261,
    KEY_RIGHT = 262,
    KEY_LEFT = 263,
    KEY_DOWN = 264,
    KEY_UP = 265,
    KEY_PAGE_UP = 266,
    KEY_PAGE_DOWN = 267,
    KEY_HOME = 268,
    KEY_END = 269,
    KEY_CAPS_LOCK = 280,
    KEY_SCROLL_LOCK = 281,
    KEY_NUM_LOCK = 282,
    KEY_PRINT_SCREEN = 283,
    KEY_PAUSE = 284,
    KEY_F1 = 290,
    KEY_F2 = 291,
    KEY_F3 = 292,
    KEY_F4 = 293,
    KEY_F5 = 294,
    KEY_F6 = 295,
    KEY_F7 = 296,
    KEY_F8 = 297,
    KEY_F9 = 298,
    KEY_F10 = 299,
    KEY_F11 = 300,
    KEY_F12 = 301,
    KEY_LEFT_SHIFT = 340,
    KEY_LEFT_CONTROL = 341,
    KEY_LEFT_ALT = 342,
    KEY_LEFT_SUPER = 343,
    KEY_RIGHT_SHIFT = 344,
    KEY_RIGHT_CONTROL = 345,
    KEY_RIGHT_ALT = 346,
    KEY_RIGHT_SUPER = 347,
    KEY_KB_MENU = 348,
    KEY_KP_0 = 320,
    KEY_KP_1 = 321,
    KEY_KP_2 = 322,
    KEY_KP_3 = 323,
    KEY_KP_4 = 324,
    KEY_KP_5 = 325,
    KEY_KP_6 = 326,
    KEY_KP_7 = 327,
    KEY_KP_8 = 328,
    KEY_KP_9 = 329,
    KEY_KP_DECIMAL = 330,
    KEY_KP_DIVIDE = 331,
    KEY_KP_MULTIPLY = 332,
    KEY_KP_SUBTRACT = 333,
    KEY_KP_ADD = 334,
    KEY_KP_ENTER = 335,
    KEY_KP_EQUAL = 336,
    /* Android key buttons */
    KEY_BACK = 4,
    KEY_MENU = 82,
    KEY_VOLUME_UP = 24,
    KEY_VOLUME_DOWN = 25
  } KeyboardKey;

  typedef enum {
    MOUSE_BUTTON_LEFT = 0,
    MOUSE_BUTTON_RIGHT = 1,
    MOUSE_BUTTON_MIDDLE = 2,
    MOUSE_BUTTON_SIDE = 3,
    MOUSE_BUTTON_EXTRA = 4,
    MOUSE_BUTTON_FORWARD = 5,
    MOUSE_BUTTON_BACK = 6
  } MouseButton;

  typedef enum {
    MOUSE_CURSOR_DEFAULT = 0,
    MOUSE_CURSOR_ARROW = 1,
    MOUSE_CURSOR_IBEAM = 2,
    MOUSE_CURSOR_CROSSHAIR = 3,
    MOUSE_CURSOR_POINTING_HAND = 4,
    MOUSE_CURSOR_RESIZE_EW = 5,
    MOUSE_CURSOR_RESIZE_NS = 6,
    MOUSE_CURSOR_RESIZE_NWSE = 7,
    MOUSE_CURSOR_RESIZE_NESW = 8,
    MOUSE_CURSOR_RESIZE_ALL = 9,
    MOUSE_CURSOR_NOT_ALLOWED = 10
  } MouseCursor;

  typedef enum {
    GAMEPAD_BUTTON_UNKNOWN = 0,
    GAMEPAD_BUTTON_LEFT_FACE_UP,
    GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
    GAMEPAD_BUTTON_LEFT_FACE_DOWN,
    GAMEPAD_BUTTON_LEFT_FACE_LEFT,
    GAMEPAD_BUTTON_RIGHT_FACE_UP,
    GAMEPAD_BUTTON_RIGHT_FACE_RIGHT,
    GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
    GAMEPAD_BUTTON_RIGHT_FACE_LEFT,
    GAMEPAD_BUTTON_LEFT_TRIGGER_1,
    GAMEPAD_BUTTON_LEFT_TRIGGER_2,
    GAMEPAD_BUTTON_RIGHT_TRIGGER_1,
    GAMEPAD_BUTTON_RIGHT_TRIGGER_2,
    GAMEPAD_BUTTON_MIDDLE_LEFT,
    GAMEPAD_BUTTON_MIDDLE,
    GAMEPAD_BUTTON_MIDDLE_RIGHT,
    GAMEPAD_BUTTON_LEFT_THUMB,
    GAMEPAD_BUTTON_RIGHT_THUMB
  } GamepadButton;

  typedef enum {
    GAMEPAD_AXIS_UNKNOWN = 0,
    GAMEPAD_AXIS_LEFT_X,
    GAMEPAD_AXIS_LEFT_Y,
    GAMEPAD_AXIS_RIGHT_X,
    GAMEPAD_AXIS_RIGHT_Y,
    GAMEPAD_AXIS_LEFT_TRIGGER,
    GAMEPAD_AXIS_RIGHT_TRIGGER
  } GamepadAxis;

  typedef enum {
    MATERIAL_MAP_ALBEDO = 0,
    MATERIAL_MAP_METALNESS,
    MATERIAL_MAP_NORMAL,
    MATERIAL_MAP_ROUGHNESS,
    MATERIAL_MAP_OCCLUSION,
    MATERIAL_MAP_EMISSION,
    MATERIAL_MAP_HEIGHT,
    MATERIAL_MAP_CUBEMAP,
    MATERIAL_MAP_IRRADIANCE,
    MATERIAL_MAP_PREFILTER,
    MATERIAL_MAP_BRDF,
  } MaterialMapIndex;

  typedef enum {
    SHADER_LOC_VERTEX_POSITION = 0,
    SHADER_LOC_VERTEX_TEXCOORD01,
    SHADER_LOC_VERTEX_TEXCOORD02,
    SHADER_LOC_VERTEX_NORMAL,
    SHADER_LOC_VERTEX_TANGENT,
    SHADER_LOC_VERTEX_COLOR,
    SHADER_LOC_MATRIX_MVP,
    SHADER_LOC_MATRIX_VIEW,
    SHADER_LOC_MATRIX_PROJECTION,
    SHADER_LOC_MATRIX_MODEL,
    SHADER_LOC_MATRIX_NORMAL,
    SHADER_LOC_VECTOR_VIEW,
    SHADER_LOC_COLOR_DIFFUSE,
    SHADER_LOC_COLOR_SPECULAR,
    SHADER_LOC_COLOR_AMBIENT,
    SHADER_LOC_MAP_ALBEDO,
    SHADER_LOC_MAP_METALNESS,
    SHADER_LOC_MAP_NORMAL,
    SHADER_LOC_MAP_ROUGHNESS,
    SHADER_LOC_MAP_OCCLUSION,
    SHADER_LOC_MAP_EMISSION,
    SHADER_LOC_MAP_HEIGHT,
    SHADER_LOC_MAP_CUBEMAP,
    SHADER_LOC_MAP_IRRADIANCE,
    SHADER_LOC_MAP_PREFILTER,
    SHADER_LOC_MAP_BRDF
  } ShaderLocationIndex;

  typedef enum {
    SHADER_UNIFORM_FLOAT = 0,
    SHADER_UNIFORM_VEC2,
    SHADER_UNIFORM_VEC3,
    SHADER_UNIFORM_VEC4,
    SHADER_UNIFORM_INT,
    SHADER_UNIFORM_IVEC2,
    SHADER_UNIFORM_IVEC3,
    SHADER_UNIFORM_IVEC4,
    SHADER_UNIFORM_SAMPLER2D
  } ShaderUniformDataType;

  typedef enum {
    SHADER_ATTRIB_FLOAT = 0,
    SHADER_ATTRIB_VEC2,
    SHADER_ATTRIB_VEC3,
    SHADER_ATTRIB_VEC4
  } ShaderAttributeDataType;

  typedef enum {
    PIXELFORMAT_UNCOMPRESSED_GRAYSCALE = 1,
    PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA,
    PIXELFORMAT_UNCOMPRESSED_R5G6B5,
    PIXELFORMAT_UNCOMPRESSED_R8G8B8,
    PIXELFORMAT_UNCOMPRESSED_R5G5B5A1,
    PIXELFORMAT_UNCOMPRESSED_R4G4B4A4,
    PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
    PIXELFORMAT_UNCOMPRESSED_R32,
    PIXELFORMAT_UNCOMPRESSED_R32G32B32,
    PIXELFORMAT_UNCOMPRESSED_R32G32B32A32,
    PIXELFORMAT_UNCOMPRESSED_R16,
    PIXELFORMAT_UNCOMPRESSED_R16G16B16,
    PIXELFORMAT_UNCOMPRESSED_R16G16B16A16,
    PIXELFORMAT_COMPRESSED_DXT1_RGB,
    PIXELFORMAT_COMPRESSED_DXT1_RGBA,
    PIXELFORMAT_COMPRESSED_DXT3_RGBA,
    PIXELFORMAT_COMPRESSED_DXT5_RGBA,
    PIXELFORMAT_COMPRESSED_ETC1_RGB,
    PIXELFORMAT_COMPRESSED_ETC2_RGB,
    PIXELFORMAT_COMPRESSED_ETC2_EAC_RGBA,
    PIXELFORMAT_COMPRESSED_PVRT_RGB,
    PIXELFORMAT_COMPRESSED_PVRT_RGBA,
    PIXELFORMAT_COMPRESSED_ASTC_4x4_RGBA,
    PIXELFORMAT_COMPRESSED_ASTC_8x8_RGBA
  } PixelFormat;

  typedef enum {
    TEXTURE_FILTER_POINT = 0,
    TEXTURE_FILTER_BILINEAR,
    TEXTURE_FILTER_TRILINEAR,
    TEXTURE_FILTER_ANISOTROPIC_4X,
    TEXTURE_FILTER_ANISOTROPIC_8X,
    TEXTURE_FILTER_ANISOTROPIC_16X,
  } TextureFilter;

  typedef enum {
    TEXTURE_WRAP_REPEAT = 0,
    TEXTURE_WRAP_CLAMP,
    TEXTURE_WRAP_MIRROR_REPEAT,
    TEXTURE_WRAP_MIRROR_CLAMP
  } TextureWrapMode;

  typedef enum {
    CUBEMAP_LAYOUT_AUTO_DETECT = 0,
    CUBEMAP_LAYOUT_LINE_VERTICAL,
    CUBEMAP_LAYOUT_LINE_HORIZONTAL,
    CUBEMAP_LAYOUT_CROSS_THREE_BY_FOUR,
    CUBEMAP_LAYOUT_CROSS_FOUR_BY_THREE,
    CUBEMAP_LAYOUT_PANORAMA
  } CubemapLayout;

  typedef enum {
    FONT_DEFAULT = 0,
    FONT_BITMAP,
    FONT_SDF
  } FontType;

  typedef enum {
    BLEND_ALPHA = 0,
    BLEND_ADDITIVE,
    BLEND_MULTIPLIED,
    BLEND_ADD_COLORS,
    BLEND_SUBTRACT_COLORS,
    BLEND_ALPHA_PREMULTIPLY,
    BLEND_CUSTOM,
    BLEND_CUSTOM_SEPARATE
  } BlendMode;

  typedef enum {
    GESTURE_NONE = 0,
    GESTURE_TAP = 1,
    GESTURE_DOUBLETAP = 2,
    GESTURE_HOLD = 4,
    GESTURE_DRAG = 8,
    GESTURE_SWIPE_RIGHT = 16,
    GESTURE_SWIPE_LEFT = 32,
    GESTURE_SWIPE_UP = 64,
    GESTURE_SWIPE_DOWN = 128,
    GESTURE_PINCH_IN = 256,
    GESTURE_PINCH_OUT = 512
  } Gesture;

  typedef enum {
    CAMERA_CUSTOM = 0,
    CAMERA_FREE,
    CAMERA_ORBITAL,
    CAMERA_FIRST_PERSON,
    CAMERA_THIRD_PERSON
  } CameraMode;

  typedef enum {
    CAMERA_PERSPECTIVE = 0,
    CAMERA_ORTHOGRAPHIC
  } CameraProjection;

  typedef enum {
    NPATCH_NINE_PATCH = 0,
    NPATCH_THREE_PATCH_VERTICAL,
    NPATCH_THREE_PATCH_HORIZONTAL
  } NPatchLayout;

  typedef void (*TraceLogCallback)(int logLevel, const char *text, va_list args);
  typedef void (*AudioCallback)(void *bufferData, unsigned int frames);
]]

-- raymath cdef
ffi.cdef [[
  typedef struct float3 { float v[3]; } float3;
  typedef struct float16 { float v[16]; } float16;
]]

-- rlgl cdef
ffi.cdef [[
  typedef enum {
    RL_OPENGL_11 = 1,
    RL_OPENGL_21,
    RL_OPENGL_33,
    RL_OPENGL_43,
    RL_OPENGL_ES_20,
    RL_OPENGL_ES_30
  } rlGlVersion;

  typedef enum {
    RL_LOG_ALL = 0,
    RL_LOG_TRACE,
    RL_LOG_DEBUG,
    RL_LOG_INFO,
    RL_LOG_WARNING,
    RL_LOG_ERROR,
    RL_LOG_FATAL,
    RL_LOG_NONE
  } rlTraceLogLevel;

  typedef enum {
    RL_PIXELFORMAT_UNCOMPRESSED_GRAYSCALE = 1,
    RL_PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA,
    RL_PIXELFORMAT_UNCOMPRESSED_R5G6B5,
    RL_PIXELFORMAT_UNCOMPRESSED_R8G8B8,
    RL_PIXELFORMAT_UNCOMPRESSED_R5G5B5A1,
    RL_PIXELFORMAT_UNCOMPRESSED_R4G4B4A4,
    RL_PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
    RL_PIXELFORMAT_UNCOMPRESSED_R32,
    RL_PIXELFORMAT_UNCOMPRESSED_R32G32B32,
    RL_PIXELFORMAT_UNCOMPRESSED_R32G32B32A32,
    RL_PIXELFORMAT_UNCOMPRESSED_R16,
    RL_PIXELFORMAT_UNCOMPRESSED_R16G16B16,
    RL_PIXELFORMAT_UNCOMPRESSED_R16G16B16A16,
    RL_PIXELFORMAT_COMPRESSED_DXT1_RGB,
    RL_PIXELFORMAT_COMPRESSED_DXT1_RGBA,
    RL_PIXELFORMAT_COMPRESSED_DXT3_RGBA,
    RL_PIXELFORMAT_COMPRESSED_DXT5_RGBA,
    RL_PIXELFORMAT_COMPRESSED_ETC1_RGB,
    RL_PIXELFORMAT_COMPRESSED_ETC2_RGB,
    RL_PIXELFORMAT_COMPRESSED_ETC2_EAC_RGBA,
    RL_PIXELFORMAT_COMPRESSED_PVRT_RGB,
    RL_PIXELFORMAT_COMPRESSED_PVRT_RGBA,
    RL_PIXELFORMAT_COMPRESSED_ASTC_4x4_RGBA,
    RL_PIXELFORMAT_COMPRESSED_ASTC_8x8_RGBA
  } rlPixelFormat;

  typedef enum {
    RL_TEXTURE_FILTER_POINT = 0,
    RL_TEXTURE_FILTER_BILINEAR,
    RL_TEXTURE_FILTER_TRILINEAR,
    RL_TEXTURE_FILTER_ANISOTROPIC_4X,
    RL_TEXTURE_FILTER_ANISOTROPIC_8X,
    RL_TEXTURE_FILTER_ANISOTROPIC_16X,
  } rlTextureFilter;

  typedef enum {
    RL_BLEND_ALPHA = 0,
    RL_BLEND_ADDITIVE,
    RL_BLEND_MULTIPLIED,
    RL_BLEND_ADD_COLORS,
    RL_BLEND_SUBTRACT_COLORS,
    RL_BLEND_ALPHA_PREMULTIPLY,
    RL_BLEND_CUSTOM
  } rlBlendMode;

  typedef enum {
    RL_SHADER_LOC_VERTEX_POSITION = 0,
    RL_SHADER_LOC_VERTEX_TEXCOORD01,
    RL_SHADER_LOC_VERTEX_TEXCOORD02,
    RL_SHADER_LOC_VERTEX_NORMAL,
    RL_SHADER_LOC_VERTEX_TANGENT,
    RL_SHADER_LOC_VERTEX_COLOR,
    RL_SHADER_LOC_MATRIX_MVP,
    RL_SHADER_LOC_MATRIX_VIEW,
    RL_SHADER_LOC_MATRIX_PROJECTION,
    RL_SHADER_LOC_MATRIX_MODEL,
    RL_SHADER_LOC_MATRIX_NORMAL,
    RL_SHADER_LOC_VECTOR_VIEW,
    RL_SHADER_LOC_COLOR_DIFFUSE,
    RL_SHADER_LOC_COLOR_SPECULAR,
    RL_SHADER_LOC_COLOR_AMBIENT,
    RL_SHADER_LOC_MAP_ALBEDO,
    RL_SHADER_LOC_MAP_METALNESS,
    RL_SHADER_LOC_MAP_NORMAL,
    RL_SHADER_LOC_MAP_ROUGHNESS,
    RL_SHADER_LOC_MAP_OCCLUSION,
    RL_SHADER_LOC_MAP_EMISSION,
    RL_SHADER_LOC_MAP_HEIGHT,
    RL_SHADER_LOC_MAP_CUBEMAP,
    RL_SHADER_LOC_MAP_IRRADIANCE,
    RL_SHADER_LOC_MAP_PREFILTER,
    RL_SHADER_LOC_MAP_BRDF
  } rlShaderLocationIndex;

  typedef enum {
    RL_SHADER_UNIFORM_FLOAT = 0,
    RL_SHADER_UNIFORM_VEC2,
    RL_SHADER_UNIFORM_VEC3,
    RL_SHADER_UNIFORM_VEC4,
    RL_SHADER_UNIFORM_INT,
    RL_SHADER_UNIFORM_IVEC2,
    RL_SHADER_UNIFORM_IVEC3,
    RL_SHADER_UNIFORM_IVEC4,
    RL_SHADER_UNIFORM_SAMPLER2D
  } rlShaderUniformDataType;

  typedef enum {
    RL_SHADER_ATTRIB_FLOAT = 0,
    RL_SHADER_ATTRIB_VEC2,
    RL_SHADER_ATTRIB_VEC3,
    RL_SHADER_ATTRIB_VEC4
  } rlShaderAttributeDataType;

  typedef enum {
    RL_ATTACHMENT_COLOR_CHANNEL0 = 0,
    RL_ATTACHMENT_COLOR_CHANNEL1 = 1,
    RL_ATTACHMENT_COLOR_CHANNEL2 = 2,
    RL_ATTACHMENT_COLOR_CHANNEL3 = 3,
    RL_ATTACHMENT_COLOR_CHANNEL4 = 4,
    RL_ATTACHMENT_COLOR_CHANNEL5 = 5,
    RL_ATTACHMENT_COLOR_CHANNEL6 = 6,
    RL_ATTACHMENT_COLOR_CHANNEL7 = 7,
    RL_ATTACHMENT_DEPTH = 100,
    RL_ATTACHMENT_STENCIL = 200,
  } rlFramebufferAttachType;

  typedef enum {
    RL_ATTACHMENT_CUBEMAP_POSITIVE_X = 0,
    RL_ATTACHMENT_CUBEMAP_NEGATIVE_X = 1,
    RL_ATTACHMENT_CUBEMAP_POSITIVE_Y = 2,
    RL_ATTACHMENT_CUBEMAP_NEGATIVE_Y = 3,
    RL_ATTACHMENT_CUBEMAP_POSITIVE_Z = 4,
    RL_ATTACHMENT_CUBEMAP_NEGATIVE_Z = 5,
    RL_ATTACHMENT_TEXTURE2D = 100,
    RL_ATTACHMENT_RENDERBUFFER = 200,
  } rlFramebufferAttachTextureType;

  typedef enum {
    RL_CULL_FACE_FRONT = 0,
    RL_CULL_FACE_BACK
  } rlCullMode;

  /* NOTE: Assumes non-ES OpenGL. */
  typedef struct rlVertexBuffer {
    int elementCount;

    float *vertices;
    float *texcoords;
    unsigned char *colors;
    unsigned int *indices;

    unsigned int vaoId;
    unsigned int vboId[4];
  } rlVertexBuffer;

  typedef struct rlDrawCall {
    int mode;
    int vertexCount;
    int vertexAlignment;
    //unsigned int vaoId;
    //unsigned int shaderId;
    unsigned int textureId;

    //Matrix projection;
    //Matrix modelview;
  } rlDrawCall;

  typedef struct rlRenderBatch {
    int bufferCount;
    int currentBuffer;
    rlVertexBuffer *vertexBuffer;

    rlDrawCall *draws;
    int drawCounter;
    float currentDepth;
  } rlRenderBatch;
]]

-- rlgl defines
rl.RL_TEXTURE_WRAP_S = 0x2802
rl.RL_TEXTURE_WRAP_T = 0x2803
rl.RL_TEXTURE_MAG_FILTER = 0x2800
rl.RL_TEXTURE_MIN_FILTER = 0x2801
rl.RL_TEXTURE_FILTER_NEAREST = 0x2600
rl.RL_TEXTURE_FILTER_LINEAR = 0x2601
rl.RL_TEXTURE_FILTER_MIP_NEAREST = 0x2700
rl.RL_TEXTURE_FILTER_NEAREST_MIP_LINEAR = 0x2702
rl.RL_TEXTURE_FILTER_LINEAR_MIP_NEAREST = 0x2701
rl.RL_TEXTURE_FILTER_MIP_LINEAR = 0x2703
rl.RL_TEXTURE_FILTER_ANISOTROPIC = 0x3000
rl.RL_TEXTURE_MIPMAP_BIAS_RATIO = 0x4000
rl.RL_TEXTURE_WRAP_REPEAT = 0x2901
rl.RL_TEXTURE_WRAP_CLAMP = 0x812F
rl.RL_TEXTURE_WRAP_MIRROR_REPEAT = 0x8370
rl.RL_TEXTURE_WRAP_MIRROR_CLAMP = 0x8742

rl.RL_MODELVIEW = 0x1700
rl.RL_PROJECTION = 0x1701
rl.RL_TEXTURE = 0x1702
rl.RL_LINES = 0x0001
rl.RL_TRIANGLES = 0x0004
rl.RL_QUADS = 0x0007

rl.RL_UNSIGNED_BYTE = 0x1401
rl.RL_FLOAT = 0x1406

rl.RL_STREAM_DRAW = 0x88E0
rl.RL_STREAM_READ = 0x88E1
rl.RL_STREAM_COPY = 0x88E2
rl.RL_STATIC_DRAW = 0x88E4
rl.RL_STATIC_READ = 0x88E5
rl.RL_STATIC_COPY = 0x88E6
rl.RL_DYNAMIC_DRAW = 0x88E8
rl.RL_DYNAMIC_READ = 0x88E9
rl.RL_DYNAMIC_COPY = 0x88EA

rl.RL_FRAGMENT_SHADER = 0x8B30
rl.RL_VERTEX_SHADER = 0x8B31
rl.RL_COMPUTE_SHADER = 0x91B9

rl.RL_ZERO = 0
rl.RL_ONE = 1
rl.RL_SRC_COLOR = 0x0300
rl.RL_ONE_MINUS_SRC_COLOR = 0x0301
rl.RL_SRC_ALPHA = 0x0302
rl.RL_ONE_MINUS_SRC_ALPHA = 0x0303
rl.RL_DST_ALPHA = 0x0304
rl.RL_ONE_MINUS_DST_ALPHA = 0x0305
rl.RL_DST_COLOR = 0x0306
rl.RL_ONE_MINUS_DST_COLOR = 0x0307
rl.RL_SRC_ALPHA_SATURATE = 0x0308
rl.RL_CONSTANT_COLOR = 0x8001
rl.RL_ONE_MINUS_CONSTANT_COLOR = 0x8002
rl.RL_CONSTANT_ALPHA = 0x8003
rl.RL_ONE_MINUS_CONSTANT_ALPHA = 0x8004
rl.RL_FUNC_ADD = 0x8006
rl.RL_MIN = 0x8007
rl.RL_MAX = 0x8008
rl.RL_FUNC_SUBTRACT = 0x800A
rl.RL_FUNC_REVERSE_SUBTRACT = 0x800B
rl.RL_BLEND_EQUATION = 0x8009
rl.RL_BLEND_EQUATION_RGB = 0x8009
rl.RL_BLEND_EQUATION_ALPHA = 0x883D
rl.RL_BLEND_DST_RGB = 0x80C8
rl.RL_BLEND_SRC_RGB = 0x80C9
rl.RL_BLEND_DST_ALPHA = 0x80CA
rl.RL_BLEND_SRC_ALPHA = 0x80CB
rl.RL_BLEND_COLOR = 0x8005

-- Physac cdef
ffi.cdef [[
  typedef enum PhysicsShapeType { PHYSICS_CIRCLE = 0, PHYSICS_POLYGON } PhysicsShapeType;
  typedef struct PhysicsBodyData *PhysicsBody;

  typedef struct Matrix2x2 {
    float m00;
    float m01;
    float m10;
    float m11;
  } Matrix2x2;

  typedef struct PhysicsVertexData {
    unsigned int vertexCount;
    Vector2 positions[24];
    Vector2 normals[24];
} PhysicsVertexData;

  typedef struct PhysicsShape {
    PhysicsShapeType type;
    PhysicsBody body;
    PhysicsVertexData vertexData;
    float radius;
    Matrix2x2 transform;
  } PhysicsShape;

  typedef struct PhysicsBodyData {
    unsigned int id;
    bool enabled;
    Vector2 position;
    Vector2 velocity;
    Vector2 force;
    float angularVelocity;
    float torque;
    float orient;
    float inertia;
    float inverseInertia;
    float mass;
    float inverseMass;
    float staticFriction;
    float dynamicFriction;
    float restitution;
    bool useGravity;
    bool isGrounded;
    bool freezeOrient;
    PhysicsShape shape;
  } PhysicsBodyData;

  typedef struct PhysicsManifoldData {
    unsigned int id;
    PhysicsBody bodyA;
    PhysicsBody bodyB;
    float penetration;
    Vector2 normal;
    Vector2 contacts[2];
    unsigned int contactsCount;
    float restitution;
    float dynamicFriction;
    float staticFriction;
  } PhysicsManifoldData, *PhysicsManifold;
]]

-- gestures cdef
ffi.cdef [[
  typedef enum {
    TOUCH_ACTION_UP = 0,
    TOUCH_ACTION_DOWN,
    TOUCH_ACTION_MOVE,
    TOUCH_ACTION_CANCEL
  } TouchAction;

  typedef struct {
    int touchAction;
    int pointCount;
    int pointerId[8]; // CONST
    Vector2 position[8]; // CONST
  } GestureEvent;
]]

-- raygui cdef
ffi.cdef [[
  typedef struct GuiStyleProp {
    unsigned short controlId;
    unsigned short propertyId;
    int propertyValue;
  } GuiStyleProp;

  typedef enum {
    GUI_STATE_NORMAL = 0,
    GUI_STATE_FOCUSED,
    GUI_STATE_PRESSED,
    GUI_STATE_DISABLED,
  } GuiControlState;

  typedef enum {
    GUI_TEXT_ALIGN_LEFT = 0,
    GUI_TEXT_ALIGN_CENTER,
    GUI_TEXT_ALIGN_RIGHT,
  } GuiTextAlignment;

  typedef enum {
    TEXT_ALIGN_TOP = 0,
    TEXT_ALIGN_MIDDLE,
    TEXT_ALIGN_BOTTOM
  } GuiTextAlignmentVertical;

  typedef enum {
    TEXT_WRAP_NONE = 0,
    TEXT_WRAP_CHAR,
    TEXT_WRAP_WORD
  } GuiTextWrapMode;

  typedef enum {
    DEFAULT = 0,
    LABEL,
    BUTTON,
    TOGGLE,
    SLIDER,
    PROGRESSBAR,
    CHECKBOX,
    COMBOBOX,
    DROPDOWNBOX,
    TEXTBOX,
    VALUEBOX,
    SPINNER,
    LISTVIEW,
    COLORPICKER,
    SCROLLBAR,
    STATUSBAR
  } GuiControl;

  typedef enum {
    BORDER_COLOR_NORMAL = 0,
    BASE_COLOR_NORMAL,
    TEXT_COLOR_NORMAL,
    BORDER_COLOR_FOCUSED,
    BASE_COLOR_FOCUSED,
    TEXT_COLOR_FOCUSED,
    BORDER_COLOR_PRESSED,
    BASE_COLOR_PRESSED,
    TEXT_COLOR_PRESSED,
    BORDER_COLOR_DISABLED,
    BASE_COLOR_DISABLED,
    TEXT_COLOR_DISABLED,
    BORDER_WIDTH,
    TEXT_PADDING,
    TEXT_ALIGNMENT,
  } GuiControlProperty;

  typedef enum {
    TEXT_SIZE = 16,
    TEXT_SPACING,
    LINE_COLOR,
    BACKGROUND_COLOR,
    TEXT_LINE_SPACING,
    TEXT_ALIGNMENT_VERTICAL,
    TEXT_WRAP_MODE
  } GuiDefaultProperty;

  typedef enum {
    GROUP_PADDING = 16,
  } GuiToggleProperty;

  typedef enum {
    SLIDER_WIDTH = 16,
    SLIDER_PADDING
  } GuiSliderProperty;

  typedef enum {
    PROGRESS_PADDING = 16,
  } GuiProgressBarProperty;

  typedef enum {
    ARROWS_SIZE = 16,
    ARROWS_VISIBLE,
    SCROLL_SLIDER_PADDING,
    SCROLL_SLIDER_SIZE,
    SCROLL_PADDING,
    SCROLL_SPEED,
  } GuiScrollBarProperty;

  typedef enum {
    CHECK_PADDING = 16
  } GuiCheckBoxProperty;

  typedef enum {
    COMBO_BUTTON_WIDTH = 16,
    COMBO_BUTTON_PADDING
  } GuiComboBoxProperty;

  typedef enum {
    ARROW_PADDING = 16,
    DROPDOWN_ITEMS_SPACING
  } GuiDropdownBoxProperty;

  typedef enum {
    TEXT_READONLY = 16
  } GuiTextBoxProperty;

  typedef enum {
    SPIN_BUTTON_WIDTH = 16,
    SPIN_BUTTON_SPACING,
  } GuiSpinnerProperty;

  typedef enum {
    SCROLLBAR_LEFT_SIDE = 0,
    SCROLLBAR_RIGHT_SIDE
  } GuiScrollBarSide;

  typedef enum {
    LIST_ITEMS_HEIGHT = 16,
    LIST_ITEMS_SPACING,
    SCROLLBAR_WIDTH,
    SCROLLBAR_SIDE,
  } GuiListViewProperty;

  typedef enum {
    COLOR_SELECTOR_SIZE = 16,
    HUEBAR_WIDTH,
    HUEBAR_PADDING,
    HUEBAR_SELECTOR_HEIGHT,
    HUEBAR_SELECTOR_OVERFLOW
  } GuiColorPickerProperty;

  typedef struct GuiTextBoxState {
    int cursor;
    int start;
    int index;
    int select;
  } GuiTextBoxState;

  typedef enum {
    ICON_NONE = 0,
    ICON_FOLDER_FILE_OPEN = 1,
    ICON_FILE_SAVE_CLASSIC = 2,
    ICON_FOLDER_OPEN = 3,
    ICON_FOLDER_SAVE = 4,
    ICON_FILE_OPEN = 5,
    ICON_FILE_SAVE = 6,
    ICON_FILE_EXPORT = 7,
    ICON_FILE_ADD = 8,
    ICON_FILE_DELETE = 9,
    ICON_FILETYPE_TEXT = 10,
    ICON_FILETYPE_AUDIO = 11,
    ICON_FILETYPE_IMAGE = 12,
    ICON_FILETYPE_PLAY = 13,
    ICON_FILETYPE_VIDEO = 14,
    ICON_FILETYPE_INFO = 15,
    ICON_FILE_COPY = 16,
    ICON_FILE_CUT = 17,
    ICON_FILE_PASTE = 18,
    ICON_CURSOR_HAND = 19,
    ICON_CURSOR_POINTER = 20,
    ICON_CURSOR_CLASSIC = 21,
    ICON_PENCIL = 22,
    ICON_PENCIL_BIG = 23,
    ICON_BRUSH_CLASSIC = 24,
    ICON_BRUSH_PAINTER = 25,
    ICON_WATER_DROP = 26,
    ICON_COLOR_PICKER = 27,
    ICON_RUBBER = 28,
    ICON_COLOR_BUCKET = 29,
    ICON_TEXT_T = 30,
    ICON_TEXT_A = 31,
    ICON_SCALE = 32,
    ICON_RESIZE = 33,
    ICON_FILTER_POINT = 34,
    ICON_FILTER_BILINEAR = 35,
    ICON_CROP = 36,
    ICON_CROP_ALPHA = 37,
    ICON_SQUARE_TOGGLE = 38,
    ICON_SYMMETRY = 39,
    ICON_SYMMETRY_HORIZONTAL = 40,
    ICON_SYMMETRY_VERTICAL = 41,
    ICON_LENS = 42,
    ICON_LENS_BIG = 43,
    ICON_EYE_ON = 44,
    ICON_EYE_OFF = 45,
    ICON_FILTER_TOP = 46,
    ICON_FILTER = 47,
    ICON_TARGET_POINT = 48,
    ICON_TARGET_SMALL = 49,
    ICON_TARGET_BIG = 50,
    ICON_TARGET_MOVE = 51,
    ICON_CURSOR_MOVE = 52,
    ICON_CURSOR_SCALE = 53,
    ICON_CURSOR_SCALE_RIGHT = 54,
    ICON_CURSOR_SCALE_LEFT = 55,
    ICON_UNDO = 56,
    ICON_REDO = 57,
    ICON_REREDO = 58,
    ICON_MUTATE = 59,
    ICON_ROTATE = 60,
    ICON_REPEAT = 61,
    ICON_SHUFFLE = 62,
    ICON_EMPTYBOX = 63,
    ICON_TARGET = 64,
    ICON_TARGET_SMALL_FILL = 65,
    ICON_TARGET_BIG_FILL = 66,
    ICON_TARGET_MOVE_FILL = 67,
    ICON_CURSOR_MOVE_FILL = 68,
    ICON_CURSOR_SCALE_FILL = 69,
    ICON_CURSOR_SCALE_RIGHT_FILL = 70,
    ICON_CURSOR_SCALE_LEFT_FILL = 71,
    ICON_UNDO_FILL = 72,
    ICON_REDO_FILL = 73,
    ICON_REREDO_FILL = 74,
    ICON_MUTATE_FILL = 75,
    ICON_ROTATE_FILL = 76,
    ICON_REPEAT_FILL = 77,
    ICON_SHUFFLE_FILL = 78,
    ICON_EMPTYBOX_SMALL = 79,
    ICON_BOX = 80,
    ICON_BOX_TOP = 81,
    ICON_BOX_TOP_RIGHT = 82,
    ICON_BOX_RIGHT = 83,
    ICON_BOX_BOTTOM_RIGHT = 84,
    ICON_BOX_BOTTOM = 85,
    ICON_BOX_BOTTOM_LEFT = 86,
    ICON_BOX_LEFT = 87,
    ICON_BOX_TOP_LEFT = 88,
    ICON_BOX_CENTER = 89,
    ICON_BOX_CIRCLE_MASK = 90,
    ICON_POT = 91,
    ICON_ALPHA_MULTIPLY = 92,
    ICON_ALPHA_CLEAR = 93,
    ICON_DITHERING = 94,
    ICON_MIPMAPS = 95,
    ICON_BOX_GRID = 96,
    ICON_GRID = 97,
    ICON_BOX_CORNERS_SMALL = 98,
    ICON_BOX_CORNERS_BIG = 99,
    ICON_FOUR_BOXES = 100,
    ICON_GRID_FILL = 101,
    ICON_BOX_MULTISIZE = 102,
    ICON_ZOOM_SMALL = 103,
    ICON_ZOOM_MEDIUM = 104,
    ICON_ZOOM_BIG = 105,
    ICON_ZOOM_ALL = 106,
    ICON_ZOOM_CENTER = 107,
    ICON_BOX_DOTS_SMALL = 108,
    ICON_BOX_DOTS_BIG = 109,
    ICON_BOX_CONCENTRIC = 110,
    ICON_BOX_GRID_BIG = 111,
    ICON_OK_TICK = 112,
    ICON_CROSS = 113,
    ICON_ARROW_LEFT = 114,
    ICON_ARROW_RIGHT = 115,
    ICON_ARROW_DOWN = 116,
    ICON_ARROW_UP = 117,
    ICON_ARROW_LEFT_FILL = 118,
    ICON_ARROW_RIGHT_FILL = 119,
    ICON_ARROW_DOWN_FILL = 120,
    ICON_ARROW_UP_FILL = 121,
    ICON_AUDIO = 122,
    ICON_FX = 123,
    ICON_WAVE = 124,
    ICON_WAVE_SINUS = 125,
    ICON_WAVE_SQUARE = 126,
    ICON_WAVE_TRIANGULAR = 127,
    ICON_CROSS_SMALL = 128,
    ICON_PLAYER_PREVIOUS = 129,
    ICON_PLAYER_PLAY_BACK = 130,
    ICON_PLAYER_PLAY = 131,
    ICON_PLAYER_PAUSE = 132,
    ICON_PLAYER_STOP = 133,
    ICON_PLAYER_NEXT = 134,
    ICON_PLAYER_RECORD = 135,
    ICON_MAGNET = 136,
    ICON_LOCK_CLOSE = 137,
    ICON_LOCK_OPEN = 138,
    ICON_CLOCK = 139,
    ICON_TOOLS = 140,
    ICON_GEAR = 141,
    ICON_GEAR_BIG = 142,
    ICON_BIN = 143,
    ICON_HAND_POINTER = 144,
    ICON_LASER = 145,
    ICON_COIN = 146,
    ICON_EXPLOSION = 147,
    ICON_1UP = 148,
    ICON_PLAYER = 149,
    ICON_PLAYER_JUMP = 150,
    ICON_KEY = 151,
    ICON_DEMON = 152,
    ICON_TEXT_POPUP = 153,
    ICON_GEAR_EX = 154,
    ICON_CRACK = 155,
    ICON_CRACK_POINTS = 156,
    ICON_STAR = 157,
    ICON_DOOR = 158,
    ICON_EXIT = 159,
    ICON_MODE_2D = 160,
    ICON_MODE_3D = 161,
    ICON_CUBE = 162,
    ICON_CUBE_FACE_TOP = 163,
    ICON_CUBE_FACE_LEFT = 164,
    ICON_CUBE_FACE_FRONT = 165,
    ICON_CUBE_FACE_BOTTOM = 166,
    ICON_CUBE_FACE_RIGHT = 167,
    ICON_CUBE_FACE_BACK = 168,
    ICON_CAMERA = 169,
    ICON_SPECIAL = 170,
    ICON_LINK_NET = 171,
    ICON_LINK_BOXES = 172,
    ICON_LINK_MULTI = 173,
    ICON_LINK = 174,
    ICON_LINK_BROKE = 175,
    ICON_TEXT_NOTES = 176,
    ICON_NOTEBOOK = 177,
    ICON_SUITCASE = 178,
    ICON_SUITCASE_ZIP = 179,
    ICON_MAILBOX = 180,
    ICON_MONITOR = 181,
    ICON_PRINTER = 182,
    ICON_PHOTO_CAMERA = 183,
    ICON_PHOTO_CAMERA_FLASH = 184,
    ICON_HOUSE = 185,
    ICON_HEART = 186,
    ICON_CORNER = 187,
    ICON_VERTICAL_BARS = 188,
    ICON_VERTICAL_BARS_FILL = 189,
    ICON_LIFE_BARS = 190,
    ICON_INFO = 191,
    ICON_CROSSLINE = 192,
    ICON_HELP = 193,
    ICON_FILETYPE_ALPHA = 194,
    ICON_FILETYPE_HOME = 195,
    ICON_LAYERS_VISIBLE = 196,
    ICON_LAYERS = 197,
    ICON_WINDOW = 198,
    ICON_HIDPI = 199,
    ICON_FILETYPE_BINARY = 200,
    ICON_HEX = 201,
    ICON_SHIELD = 202,
    ICON_FILE_NEW = 203,
    ICON_FOLDER_ADD = 204,
    ICON_ALARM = 205,
    ICON_CPU = 206,
    ICON_ROM = 207,
    ICON_STEP_OVER = 208,
    ICON_STEP_INTO = 209,
    ICON_STEP_OUT = 210,
    ICON_RESTART = 211,
    ICON_BREAKPOINT_ON = 212,
    ICON_BREAKPOINT_OFF = 213,
    ICON_BURGER_MENU = 214,
    ICON_CASE_SENSITIVE = 215,
    ICON_REG_EXP = 216,
    ICON_FOLDER = 217,
    ICON_FILE = 218,
    ICON_219 = 219,
    ICON_220 = 220,
    ICON_221 = 221,
    ICON_222 = 222,
    ICON_223 = 223,
    ICON_224 = 224,
    ICON_225 = 225,
    ICON_226 = 226,
    ICON_227 = 227,
    ICON_228 = 228,
    ICON_229 = 229,
    ICON_230 = 230,
    ICON_231 = 231,
    ICON_232 = 232,
    ICON_233 = 233,
    ICON_234 = 234,
    ICON_235 = 235,
    ICON_236 = 236,
    ICON_237 = 237,
    ICON_238 = 238,
    ICON_239 = 239,
    ICON_240 = 240,
    ICON_241 = 241,
    ICON_242 = 242,
    ICON_243 = 243,
    ICON_244 = 244,
    ICON_245 = 245,
    ICON_246 = 246,
    ICON_247 = 247,
    ICON_248 = 248,
    ICON_249 = 249,
    ICON_250 = 250,
    ICON_251 = 251,
    ICON_252 = 252,
    ICON_253 = 253,
    ICON_254 = 254,
    ICON_255 = 255,
  } GuiIconName;
typedef void (*TraceLogCallback)(int logLevel, const char *text, va_list args);  // Logging: Redirect trace log messages
typedef unsigned char *(*LoadFileDataCallback)(const char *fileName, int *dataSize);    // FileIO: Load binary data
typedef bool (*SaveFileDataCallback)(const char *fileName, void *data, int dataSize);   // FileIO: Save binary data
typedef char *(*LoadFileTextCallback)(const char *fileName);            // FileIO: Load text data
typedef bool (*SaveFileTextCallback)(const char *fileName, char *text); // FileIO: Save text data
void InitWindow(int width, int height, const char *title);  // Initialize window and OpenGL context
void CloseWindow(void);                                     // Close window and unload OpenGL context
bool WindowShouldClose(void);                               // Check if application should close (KEY_ESCAPE pressed or windows close icon clicked)
bool IsWindowReady(void);                                   // Check if window has been initialized successfully
bool IsWindowFullscreen(void);                              // Check if window is currently fullscreen
bool IsWindowHidden(void);                                  // Check if window is currently hidden
bool IsWindowMinimized(void);                               // Check if window is currently minimized
bool IsWindowMaximized(void);                               // Check if window is currently maximized
bool IsWindowFocused(void);                                 // Check if window is currently focused
bool IsWindowResized(void);                                 // Check if window has been resized last frame
bool IsWindowState(unsigned int flag);                      // Check if one specific window flag is enabled
void SetWindowState(unsigned int flags);                    // Set window configuration state using flags
void ClearWindowState(unsigned int flags);                  // Clear window configuration state flags
void ToggleFullscreen(void);                                // Toggle window state: fullscreen/windowed, resizes monitor to match window resolution
void ToggleBorderlessWindowed(void);                        // Toggle window state: borderless windowed, resizes window to match monitor resolution
void MaximizeWindow(void);                                  // Set window state: maximized, if resizable
void MinimizeWindow(void);                                  // Set window state: minimized, if resizable
void RestoreWindow(void);                                   // Set window state: not minimized/maximized
void SetWindowIcon(Image image);                            // Set icon for window (single image, RGBA 32bit)
void SetWindowIcons(Image *images, int count);              // Set icon for window (multiple images, RGBA 32bit)
void SetWindowTitle(const char *title);                     // Set title for window
void SetWindowPosition(int x, int y);                       // Set window position on screen
void SetWindowMonitor(int monitor);                         // Set monitor for the current window
void SetWindowMinSize(int width, int height);               // Set window minimum dimensions (for FLAG_WINDOW_RESIZABLE)
void SetWindowMaxSize(int width, int height);               // Set window maximum dimensions (for FLAG_WINDOW_RESIZABLE)
void SetWindowSize(int width, int height);                  // Set window dimensions
void SetWindowOpacity(float opacity);                       // Set window opacity [0.0f..1.0f]
void SetWindowFocused(void);                                // Set window focused
void *GetWindowHandle(void);                                // Get native window handle
int GetScreenWidth(void);                                   // Get current screen width
int GetScreenHeight(void);                                  // Get current screen height
int GetRenderWidth(void);                                   // Get current render width (it considers HiDPI)
int GetRenderHeight(void);                                  // Get current render height (it considers HiDPI)
int GetMonitorCount(void);                                  // Get number of connected monitors
int GetCurrentMonitor(void);                                // Get current monitor where window is placed
Vector2 GetMonitorPosition(int monitor);                    // Get specified monitor position
int GetMonitorWidth(int monitor);                           // Get specified monitor width (current video mode used by monitor)
int GetMonitorHeight(int monitor);                          // Get specified monitor height (current video mode used by monitor)
int GetMonitorPhysicalWidth(int monitor);                   // Get specified monitor physical width in millimetres
int GetMonitorPhysicalHeight(int monitor);                  // Get specified monitor physical height in millimetres
int GetMonitorRefreshRate(int monitor);                     // Get specified monitor refresh rate
Vector2 GetWindowPosition(void);                            // Get window position XY on monitor
Vector2 GetWindowScaleDPI(void);                            // Get window scale DPI factor
const char *GetMonitorName(int monitor);                    // Get the human-readable, UTF-8 encoded name of the specified monitor
void SetClipboardText(const char *text);                    // Set clipboard text content
const char *GetClipboardText(void);                         // Get clipboard text content
Image GetClipboardImage(void);                              // Get clipboard image content
void EnableEventWaiting(void);                              // Enable waiting for events on EndDrawing(), no automatic event polling
void DisableEventWaiting(void);                             // Disable waiting for events on EndDrawing(), automatic events polling

// Cursor-related functions
void ShowCursor(void);                                      // Shows cursor
void HideCursor(void);                                      // Hides cursor
bool IsCursorHidden(void);                                  // Check if cursor is not visible
void EnableCursor(void);                                    // Enables cursor (unlock cursor)
void DisableCursor(void);                                   // Disables cursor (lock cursor)
bool IsCursorOnScreen(void);                                // Check if cursor is on the screen

// Drawing-related functions
void ClearBackground(Color color);                          // Set background color (framebuffer clear color)
void BeginDrawing(void);                                    // Setup canvas (framebuffer) to start drawing
void EndDrawing(void);                                      // End canvas drawing and swap buffers (double buffering)
void BeginMode2D(Camera2D camera);                          // Begin 2D mode with custom camera (2D)
void EndMode2D(void);                                       // Ends 2D mode with custom camera
void BeginMode3D(Camera3D camera);                          // Begin 3D mode with custom camera (3D)
void EndMode3D(void);                                       // Ends 3D mode and returns to default 2D orthographic mode
void BeginTextureMode(RenderTexture2D target);              // Begin drawing to render texture
void EndTextureMode(void);                                  // Ends drawing to render texture
void BeginShaderMode(Shader shader);                        // Begin custom shader drawing
void EndShaderMode(void);                                   // End custom shader drawing (use default shader)
void BeginBlendMode(int mode);                              // Begin blending mode (alpha, additive, multiplied, subtract, custom)
void EndBlendMode(void);                                    // End blending mode (reset to default: alpha blending)
void BeginScissorMode(int x, int y, int width, int height); // Begin scissor mode (define screen area for following drawing)
void EndScissorMode(void);                                  // End scissor mode
void BeginVrStereoMode(VrStereoConfig config);              // Begin stereo rendering (requires VR simulator)
void EndVrStereoMode(void);                                 // End stereo rendering (requires VR simulator)

VrStereoConfig LoadVrStereoConfig(VrDeviceInfo device);     // Load VR stereo config for VR simulator device parameters
void UnloadVrStereoConfig(VrStereoConfig config);           // Unload VR stereo config

// Shader management functions
// NOTE: Shader functionality is not available on OpenGL 1.1
Shader LoadShader(const char *vsFileName, const char *fsFileName);   // Load shader from files and bind default locations
Shader LoadShaderFromMemory(const char *vsCode, const char *fsCode); // Load shader from code strings and bind default locations
bool IsShaderValid(Shader shader);                                   // Check if a shader is valid (loaded on GPU)
int GetShaderLocation(Shader shader, const char *uniformName);       // Get shader uniform location
int GetShaderLocationAttrib(Shader shader, const char *attribName);  // Get shader attribute location
void SetShaderValue(Shader shader, int locIndex, const void *value, int uniformType);               // Set shader uniform value
void SetShaderValueV(Shader shader, int locIndex, const void *value, int uniformType, int count);   // Set shader uniform value vector
void SetShaderValueMatrix(Shader shader, int locIndex, Matrix mat);         // Set shader uniform value (matrix 4x4)
void SetShaderValueTexture(Shader shader, int locIndex, Texture2D texture); // Set shader uniform value for texture (sampler2d)
void UnloadShader(Shader shader);                                    // Unload shader from GPU memory (VRAM)

// Screen-space-related functions
Ray GetScreenToWorldRay(Vector2 position, Camera camera);         // Get a ray trace from screen position (i.e mouse)
Ray GetScreenToWorldRayEx(Vector2 position, Camera camera, int width, int height); // Get a ray trace from screen position (i.e mouse) in a viewport
Vector2 GetWorldToScreen(Vector3 position, Camera camera);        // Get the screen space position for a 3d world space position
Vector2 GetWorldToScreenEx(Vector3 position, Camera camera, int width, int height); // Get size position for a 3d world space position
Vector2 GetWorldToScreen2D(Vector2 position, Camera2D camera);    // Get the screen space position for a 2d camera world space position
Vector2 GetScreenToWorld2D(Vector2 position, Camera2D camera);    // Get the world space position for a 2d camera screen space position
Matrix GetCameraMatrix(Camera camera);                            // Get camera transform matrix (view matrix)
Matrix GetCameraMatrix2D(Camera2D camera);                        // Get camera 2d transform matrix

// Timing-related functions
void SetTargetFPS(int fps);                                 // Set target FPS (maximum)
float GetFrameTime(void);                                   // Get time in seconds for last frame drawn (delta time)
double GetTime(void);                                       // Get elapsed time in seconds since InitWindow()
int GetFPS(void);                                           // Get current FPS

// Custom frame control functions
// NOTE: Those functions are intended for advanced users that want full control over the frame processing
// By default EndDrawing() does this job: draws everything + SwapScreenBuffer() + manage frame timing + PollInputEvents()
// To avoid that behaviour and control frame processes manually, enable in config.h: SUPPORT_CUSTOM_FRAME_CONTROL
void SwapScreenBuffer(void);                                // Swap back buffer with front buffer (screen drawing)
void PollInputEvents(void);                                 // Register all input events
void WaitTime(double seconds);                              // Wait for some time (halt program execution)

// Random values generation functions
void SetRandomSeed(unsigned int seed);                      // Set the seed for the random number generator
int GetRandomValue(int min, int max);                       // Get a random value between min and max (both included)
int *LoadRandomSequence(unsigned int count, int min, int max); // Load random values sequence, no values repeated
void UnloadRandomSequence(int *sequence);                   // Unload random values sequence

// Misc. functions
void TakeScreenshot(const char *fileName);                  // Takes a screenshot of current screen (filename extension defines format)
void SetConfigFlags(unsigned int flags);                    // Setup init configuration flags (view FLAGS)
void OpenURL(const char *url);                              // Open URL with default system browser (if available)

// NOTE: Following functions implemented in module [utils]
//------------------------------------------------------------------
void TraceLog(int logLevel, const char *text, ...);         // Show trace log messages (LOG_DEBUG, LOG_INFO, LOG_WARNING, LOG_ERROR...)
void SetTraceLogLevel(int logLevel);                        // Set the current threshold (minimum) log level
void *MemAlloc(unsigned int size);                          // Internal memory allocator
void *MemRealloc(void *ptr, unsigned int size);             // Internal memory reallocator
void MemFree(void *ptr);                                    // Internal memory free

// Set custom callbacks
// WARNING: Callbacks setup is intended for advanced users
void SetTraceLogCallback(TraceLogCallback callback);         // Set custom trace log
void SetLoadFileDataCallback(LoadFileDataCallback callback); // Set custom file binary data loader
void SetSaveFileDataCallback(SaveFileDataCallback callback); // Set custom file binary data saver
void SetLoadFileTextCallback(LoadFileTextCallback callback); // Set custom file text data loader
void SetSaveFileTextCallback(SaveFileTextCallback callback); // Set custom file text data saver

// Files management functions
unsigned char *LoadFileData(const char *fileName, int *dataSize); // Load file data as byte array (read)
void UnloadFileData(unsigned char *data);                   // Unload file data allocated by LoadFileData()
bool SaveFileData(const char *fileName, void *data, int dataSize); // Save data to file from byte array (write), returns true on success
bool ExportDataAsCode(const unsigned char *data, int dataSize, const char *fileName); // Export data to code (.h), returns true on success
char *LoadFileText(const char *fileName);                   // Load text data from file (read), returns a '\0' terminated string
void UnloadFileText(char *text);                            // Unload file text data allocated by LoadFileText()
bool SaveFileText(const char *fileName, char *text);        // Save text data to file (write), string must be '\0' terminated, returns true on success
//------------------------------------------------------------------

// File system functions
bool FileExists(const char *fileName);                      // Check if file exists
bool DirectoryExists(const char *dirPath);                  // Check if a directory path exists
bool IsFileExtension(const char *fileName, const char *ext); // Check file extension (including point: .png, .wav)
int GetFileLength(const char *fileName);                    // Get file length in bytes (NOTE: GetFileSize() conflicts with windows.h)
const char *GetFileExtension(const char *fileName);         // Get pointer to extension for a filename string (includes dot: '.png')
const char *GetFileName(const char *filePath);              // Get pointer to filename for a path string
const char *GetFileNameWithoutExt(const char *filePath);    // Get filename string without extension (uses static string)
const char *GetDirectoryPath(const char *filePath);         // Get full path for a given fileName with path (uses static string)
const char *GetPrevDirectoryPath(const char *dirPath);      // Get previous directory path for a given path (uses static string)
const char *GetWorkingDirectory(void);                      // Get current working directory (uses static string)
const char *GetApplicationDirectory(void);                  // Get the directory of the running application (uses static string)
int MakeDirectory(const char *dirPath);                     // Create directories (including full path requested), returns 0 on success
bool ChangeDirectory(const char *dir);                      // Change working directory, return true on success
bool IsPathFile(const char *path);                          // Check if a given path is a file or a directory
bool IsFileNameValid(const char *fileName);                 // Check if fileName is valid for the platform/OS
FilePathList LoadDirectoryFiles(const char *dirPath);       // Load directory filepaths
FilePathList LoadDirectoryFilesEx(const char *basePath, const char *filter, bool scanSubdirs); // Load directory filepaths with extension filtering and recursive directory scan. Use 'DIR' in the filter string to include directories in the result
void UnloadDirectoryFiles(FilePathList files);              // Unload filepaths
bool IsFileDropped(void);                                   // Check if a file has been dropped into window
FilePathList LoadDroppedFiles(void);                        // Load dropped filepaths
void UnloadDroppedFiles(FilePathList files);                // Unload dropped filepaths
long GetFileModTime(const char *fileName);                  // Get file modification time (last write time)

// Compression/Encoding functionality
unsigned char *CompressData(const unsigned char *data, int dataSize, int *compDataSize);        // Compress data (DEFLATE algorithm), memory must be MemFree()
unsigned char *DecompressData(const unsigned char *compData, int compDataSize, int *dataSize);  // Decompress data (DEFLATE algorithm), memory must be MemFree()
char *EncodeDataBase64(const unsigned char *data, int dataSize, int *outputSize);               // Encode data to Base64 string, memory must be MemFree()
unsigned char *DecodeDataBase64(const unsigned char *data, int *outputSize);                    // Decode Base64 string data, memory must be MemFree()
unsigned int ComputeCRC32(unsigned char *data, int dataSize);     // Compute CRC32 hash code
unsigned int *ComputeMD5(unsigned char *data, int dataSize);      // Compute MD5 hash code, returns static int[4] (16 bytes)
unsigned int *ComputeSHA1(unsigned char *data, int dataSize);      // Compute SHA1 hash code, returns static int[5] (20 bytes)


// Automation events functionality
AutomationEventList LoadAutomationEventList(const char *fileName);                // Load automation events list from file, NULL for empty list, capacity = MAX_AUTOMATION_EVENTS
void UnloadAutomationEventList(AutomationEventList list);                         // Unload automation events list from file
bool ExportAutomationEventList(AutomationEventList list, const char *fileName);   // Export automation events list as text file
void SetAutomationEventList(AutomationEventList *list);                           // Set automation event list to record to
void SetAutomationEventBaseFrame(int frame);                                      // Set automation event internal base frame to start recording
void StartAutomationEventRecording(void);                                         // Start recording automation events (AutomationEventList must be set)
void StopAutomationEventRecording(void);                                          // Stop recording automation events
void PlayAutomationEvent(AutomationEvent event);                                  // Play a recorded automation event

//------------------------------------------------------------------------------------
// Input Handling Functions (Module: core)
//------------------------------------------------------------------------------------

// Input-related functions: keyboard
bool IsKeyPressed(int key);                             // Check if a key has been pressed once
bool IsKeyPressedRepeat(int key);                       // Check if a key has been pressed again
bool IsKeyDown(int key);                                // Check if a key is being pressed
bool IsKeyReleased(int key);                            // Check if a key has been released once
bool IsKeyUp(int key);                                  // Check if a key is NOT being pressed
int GetKeyPressed(void);                                // Get key pressed (keycode), call it multiple times for keys queued, returns 0 when the queue is empty
int GetCharPressed(void);                               // Get char pressed (unicode), call it multiple times for chars queued, returns 0 when the queue is empty
void SetExitKey(int key);                               // Set a custom key to exit program (default is ESC)

// Input-related functions: gamepads
bool IsGamepadAvailable(int gamepad);                                        // Check if a gamepad is available
const char *GetGamepadName(int gamepad);                                     // Get gamepad internal name id
bool IsGamepadButtonPressed(int gamepad, int button);                        // Check if a gamepad button has been pressed once
bool IsGamepadButtonDown(int gamepad, int button);                           // Check if a gamepad button is being pressed
bool IsGamepadButtonReleased(int gamepad, int button);                       // Check if a gamepad button has been released once
bool IsGamepadButtonUp(int gamepad, int button);                             // Check if a gamepad button is NOT being pressed
int GetGamepadButtonPressed(void);                                           // Get the last gamepad button pressed
int GetGamepadAxisCount(int gamepad);                                        // Get gamepad axis count for a gamepad
float GetGamepadAxisMovement(int gamepad, int axis);                         // Get axis movement value for a gamepad axis
int SetGamepadMappings(const char *mappings);                                // Set internal gamepad mappings (SDL_GameControllerDB)
void SetGamepadVibration(int gamepad, float leftMotor, float rightMotor, float duration); // Set gamepad vibration for both motors (duration in seconds)

// Input-related functions: mouse
bool IsMouseButtonPressed(int button);                  // Check if a mouse button has been pressed once
bool IsMouseButtonDown(int button);                     // Check if a mouse button is being pressed
bool IsMouseButtonReleased(int button);                 // Check if a mouse button has been released once
bool IsMouseButtonUp(int button);                       // Check if a mouse button is NOT being pressed
int GetMouseX(void);                                    // Get mouse position X
int GetMouseY(void);                                    // Get mouse position Y
Vector2 GetMousePosition(void);                         // Get mouse position XY
Vector2 GetMouseDelta(void);                            // Get mouse delta between frames
void SetMousePosition(int x, int y);                    // Set mouse position XY
void SetMouseOffset(int offsetX, int offsetY);          // Set mouse offset
void SetMouseScale(float scaleX, float scaleY);         // Set mouse scaling
float GetMouseWheelMove(void);                          // Get mouse wheel movement for X or Y, whichever is larger
Vector2 GetMouseWheelMoveV(void);                       // Get mouse wheel movement for both X and Y
void SetMouseCursor(int cursor);                        // Set mouse cursor

// Input-related functions: touch
int GetTouchX(void);                                    // Get touch position X for touch point 0 (relative to screen size)
int GetTouchY(void);                                    // Get touch position Y for touch point 0 (relative to screen size)
Vector2 GetTouchPosition(int index);                    // Get touch position XY for a touch point index (relative to screen size)
int GetTouchPointId(int index);                         // Get touch point identifier for given index
int GetTouchPointCount(void);                           // Get number of touch points

//------------------------------------------------------------------------------------
// Gestures and Touch Handling Functions (Module: rgestures)
//------------------------------------------------------------------------------------
void SetGesturesEnabled(unsigned int flags);      // Enable a set of gestures using flags
bool IsGestureDetected(unsigned int gesture);     // Check if a gesture have been detected
int GetGestureDetected(void);                     // Get latest detected gesture
float GetGestureHoldDuration(void);               // Get gesture hold time in seconds
Vector2 GetGestureDragVector(void);               // Get gesture drag vector
float GetGestureDragAngle(void);                  // Get gesture drag angle
Vector2 GetGesturePinchVector(void);              // Get gesture pinch delta
float GetGesturePinchAngle(void);                 // Get gesture pinch angle

//------------------------------------------------------------------------------------
// Camera System Functions (Module: rcamera)
//------------------------------------------------------------------------------------
void UpdateCamera(Camera *camera, int mode);      // Update camera position for selected mode
void UpdateCameraPro(Camera *camera, Vector3 movement, Vector3 rotation, float zoom); // Update camera movement/rotation

//------------------------------------------------------------------------------------
// Basic Shapes Drawing Functions (Module: shapes)
//------------------------------------------------------------------------------------
// Set texture and rectangle to be used on shapes drawing
// NOTE: It can be useful when using basic shapes and one single font,
// defining a font char white rectangle would allow drawing everything in a single draw call
void SetShapesTexture(Texture2D texture, Rectangle source);       // Set texture and rectangle to be used on shapes drawing
Texture2D GetShapesTexture(void);                                 // Get texture that is used for shapes drawing
Rectangle GetShapesTextureRectangle(void);                        // Get texture source rectangle that is used for shapes drawing

// Basic shapes drawing functions
void DrawPixel(int posX, int posY, Color color);                                                   // Draw a pixel using geometry [Can be slow, use with care]
void DrawPixelV(Vector2 position, Color color);                                                    // Draw a pixel using geometry (Vector version) [Can be slow, use with care]
void DrawLine(int startPosX, int startPosY, int endPosX, int endPosY, Color color);                // Draw a line
void DrawLineV(Vector2 startPos, Vector2 endPos, Color color);                                     // Draw a line (using gl lines)
void DrawLineEx(Vector2 startPos, Vector2 endPos, float thick, Color color);                       // Draw a line (using triangles/quads)
void DrawLineStrip(const Vector2 *points, int pointCount, Color color);                            // Draw lines sequence (using gl lines)
void DrawLineBezier(Vector2 startPos, Vector2 endPos, float thick, Color color);                   // Draw line segment cubic-bezier in-out interpolation
void DrawCircle(int centerX, int centerY, float radius, Color color);                              // Draw a color-filled circle
void DrawCircleSector(Vector2 center, float radius, float startAngle, float endAngle, int segments, Color color);      // Draw a piece of a circle
void DrawCircleSectorLines(Vector2 center, float radius, float startAngle, float endAngle, int segments, Color color); // Draw circle sector outline
void DrawCircleGradient(int centerX, int centerY, float radius, Color inner, Color outer);         // Draw a gradient-filled circle
void DrawCircleV(Vector2 center, float radius, Color color);                                       // Draw a color-filled circle (Vector version)
void DrawCircleLines(int centerX, int centerY, float radius, Color color);                         // Draw circle outline
void DrawCircleLinesV(Vector2 center, float radius, Color color);                                  // Draw circle outline (Vector version)
void DrawEllipse(int centerX, int centerY, float radiusH, float radiusV, Color color);             // Draw ellipse
void DrawEllipseLines(int centerX, int centerY, float radiusH, float radiusV, Color color);        // Draw ellipse outline
void DrawRing(Vector2 center, float innerRadius, float outerRadius, float startAngle, float endAngle, int segments, Color color); // Draw ring
void DrawRingLines(Vector2 center, float innerRadius, float outerRadius, float startAngle, float endAngle, int segments, Color color);    // Draw ring outline
void DrawRectangle(int posX, int posY, int width, int height, Color color);                        // Draw a color-filled rectangle
void DrawRectangleV(Vector2 position, Vector2 size, Color color);                                  // Draw a color-filled rectangle (Vector version)
void DrawRectangleRec(Rectangle rec, Color color);                                                 // Draw a color-filled rectangle
void DrawRectanglePro(Rectangle rec, Vector2 origin, float rotation, Color color);                 // Draw a color-filled rectangle with pro parameters
void DrawRectangleGradientV(int posX, int posY, int width, int height, Color top, Color bottom);   // Draw a vertical-gradient-filled rectangle
void DrawRectangleGradientH(int posX, int posY, int width, int height, Color left, Color right);   // Draw a horizontal-gradient-filled rectangle
void DrawRectangleGradientEx(Rectangle rec, Color topLeft, Color bottomLeft, Color topRight, Color bottomRight); // Draw a gradient-filled rectangle with custom vertex colors
void DrawRectangleLines(int posX, int posY, int width, int height, Color color);                   // Draw rectangle outline
void DrawRectangleLinesEx(Rectangle rec, float lineThick, Color color);                            // Draw rectangle outline with extended parameters
void DrawRectangleRounded(Rectangle rec, float roundness, int segments, Color color);              // Draw rectangle with rounded edges
void DrawRectangleRoundedLines(Rectangle rec, float roundness, int segments, Color color);         // Draw rectangle lines with rounded edges
void DrawRectangleRoundedLinesEx(Rectangle rec, float roundness, int segments, float lineThick, Color color); // Draw rectangle with rounded edges outline
void DrawTriangle(Vector2 v1, Vector2 v2, Vector2 v3, Color color);                                // Draw a color-filled triangle (vertex in counter-clockwise order!)
void DrawTriangleLines(Vector2 v1, Vector2 v2, Vector2 v3, Color color);                           // Draw triangle outline (vertex in counter-clockwise order!)
void DrawTriangleFan(const Vector2 *points, int pointCount, Color color);                          // Draw a triangle fan defined by points (first vertex is the center)
void DrawTriangleStrip(const Vector2 *points, int pointCount, Color color);                        // Draw a triangle strip defined by points
void DrawPoly(Vector2 center, int sides, float radius, float rotation, Color color);               // Draw a regular polygon (Vector version)
void DrawPolyLines(Vector2 center, int sides, float radius, float rotation, Color color);          // Draw a polygon outline of n sides
void DrawPolyLinesEx(Vector2 center, int sides, float radius, float rotation, float lineThick, Color color); // Draw a polygon outline of n sides with extended parameters

// Splines drawing functions
void DrawSplineLinear(const Vector2 *points, int pointCount, float thick, Color color);                  // Draw spline: Linear, minimum 2 points
void DrawSplineBasis(const Vector2 *points, int pointCount, float thick, Color color);                   // Draw spline: B-Spline, minimum 4 points
void DrawSplineCatmullRom(const Vector2 *points, int pointCount, float thick, Color color);              // Draw spline: Catmull-Rom, minimum 4 points
void DrawSplineBezierQuadratic(const Vector2 *points, int pointCount, float thick, Color color);         // Draw spline: Quadratic Bezier, minimum 3 points (1 control point): [p1, c2, p3, c4...]
void DrawSplineBezierCubic(const Vector2 *points, int pointCount, float thick, Color color);             // Draw spline: Cubic Bezier, minimum 4 points (2 control points): [p1, c2, c3, p4, c5, c6...]
void DrawSplineSegmentLinear(Vector2 p1, Vector2 p2, float thick, Color color);                    // Draw spline segment: Linear, 2 points
void DrawSplineSegmentBasis(Vector2 p1, Vector2 p2, Vector2 p3, Vector2 p4, float thick, Color color); // Draw spline segment: B-Spline, 4 points
void DrawSplineSegmentCatmullRom(Vector2 p1, Vector2 p2, Vector2 p3, Vector2 p4, float thick, Color color); // Draw spline segment: Catmull-Rom, 4 points
void DrawSplineSegmentBezierQuadratic(Vector2 p1, Vector2 c2, Vector2 p3, float thick, Color color); // Draw spline segment: Quadratic Bezier, 2 points, 1 control point
void DrawSplineSegmentBezierCubic(Vector2 p1, Vector2 c2, Vector2 c3, Vector2 p4, float thick, Color color); // Draw spline segment: Cubic Bezier, 2 points, 2 control points

// Spline segment point evaluation functions, for a given t [0.0f .. 1.0f]
Vector2 GetSplinePointLinear(Vector2 startPos, Vector2 endPos, float t);                           // Get (evaluate) spline point: Linear
Vector2 GetSplinePointBasis(Vector2 p1, Vector2 p2, Vector2 p3, Vector2 p4, float t);              // Get (evaluate) spline point: B-Spline
Vector2 GetSplinePointCatmullRom(Vector2 p1, Vector2 p2, Vector2 p3, Vector2 p4, float t);         // Get (evaluate) spline point: Catmull-Rom
Vector2 GetSplinePointBezierQuad(Vector2 p1, Vector2 c2, Vector2 p3, float t);                     // Get (evaluate) spline point: Quadratic Bezier
Vector2 GetSplinePointBezierCubic(Vector2 p1, Vector2 c2, Vector2 c3, Vector2 p4, float t);        // Get (evaluate) spline point: Cubic Bezier

// Basic shapes collision detection functions
bool CheckCollisionRecs(Rectangle rec1, Rectangle rec2);                                           // Check collision between two rectangles
bool CheckCollisionCircles(Vector2 center1, float radius1, Vector2 center2, float radius2);        // Check collision between two circles
bool CheckCollisionCircleRec(Vector2 center, float radius, Rectangle rec);                         // Check collision between circle and rectangle
bool CheckCollisionCircleLine(Vector2 center, float radius, Vector2 p1, Vector2 p2);               // Check if circle collides with a line created betweeen two points [p1] and [p2]
bool CheckCollisionPointRec(Vector2 point, Rectangle rec);                                         // Check if point is inside rectangle
bool CheckCollisionPointCircle(Vector2 point, Vector2 center, float radius);                       // Check if point is inside circle
bool CheckCollisionPointTriangle(Vector2 point, Vector2 p1, Vector2 p2, Vector2 p3);               // Check if point is inside a triangle
bool CheckCollisionPointLine(Vector2 point, Vector2 p1, Vector2 p2, int threshold);                // Check if point belongs to line created between two points [p1] and [p2] with defined margin in pixels [threshold]
bool CheckCollisionPointPoly(Vector2 point, const Vector2 *points, int pointCount);                // Check if point is within a polygon described by array of vertices
bool CheckCollisionLines(Vector2 startPos1, Vector2 endPos1, Vector2 startPos2, Vector2 endPos2, Vector2 *collisionPoint); // Check the collision between two lines defined by two points each, returns collision point by reference
Rectangle GetCollisionRec(Rectangle rec1, Rectangle rec2);                                         // Get collision rectangle for two rectangles collision

//------------------------------------------------------------------------------------
// Texture Loading and Drawing Functions (Module: textures)
//------------------------------------------------------------------------------------

// Image loading functions
// NOTE: These functions do not require GPU access
Image LoadImage(const char *fileName);                                                             // Load image from file into CPU memory (RAM)
Image LoadImageRaw(const char *fileName, int width, int height, int format, int headerSize);       // Load image from RAW file data
Image LoadImageAnim(const char *fileName, int *frames);                                            // Load image sequence from file (frames appended to image.data)
Image LoadImageAnimFromMemory(const char *fileType, const unsigned char *fileData, int dataSize, int *frames); // Load image sequence from memory buffer
Image LoadImageFromMemory(const char *fileType, const unsigned char *fileData, int dataSize);      // Load image from memory buffer, fileType refers to extension: i.e. '.png'
Image LoadImageFromTexture(Texture2D texture);                                                     // Load image from GPU texture data
Image LoadImageFromScreen(void);                                                                   // Load image from screen buffer and (screenshot)
bool IsImageValid(Image image);                                                                    // Check if an image is valid (data and parameters)
void UnloadImage(Image image);                                                                     // Unload image from CPU memory (RAM)
bool ExportImage(Image image, const char *fileName);                                               // Export image data to file, returns true on success
unsigned char *ExportImageToMemory(Image image, const char *fileType, int *fileSize);              // Export image to memory buffer
bool ExportImageAsCode(Image image, const char *fileName);                                         // Export image as code file defining an array of bytes, returns true on success

// Image generation functions
Image GenImageColor(int width, int height, Color color);                                           // Generate image: plain color
Image GenImageGradientLinear(int width, int height, int direction, Color start, Color end);        // Generate image: linear gradient, direction in degrees [0..360], 0=Vertical gradient
Image GenImageGradientRadial(int width, int height, float density, Color inner, Color outer);      // Generate image: radial gradient
Image GenImageGradientSquare(int width, int height, float density, Color inner, Color outer);      // Generate image: square gradient
Image GenImageChecked(int width, int height, int checksX, int checksY, Color col1, Color col2);    // Generate image: checked
Image GenImageWhiteNoise(int width, int height, float factor);                                     // Generate image: white noise
Image GenImagePerlinNoise(int width, int height, int offsetX, int offsetY, float scale);           // Generate image: perlin noise
Image GenImageCellular(int width, int height, int tileSize);                                       // Generate image: cellular algorithm, bigger tileSize means bigger cells
Image GenImageText(int width, int height, const char *text);                                       // Generate image: grayscale image from text data

// Image manipulation functions
Image ImageCopy(Image image);                                                                      // Create an image duplicate (useful for transformations)
Image ImageFromImage(Image image, Rectangle rec);                                                  // Create an image from another image piece
Image ImageFromChannel(Image image, int selectedChannel);                                          // Create an image from a selected channel of another image (GRAYSCALE)
Image ImageText(const char *text, int fontSize, Color color);                                      // Create an image from text (default font)
Image ImageTextEx(Font font, const char *text, float fontSize, float spacing, Color tint);         // Create an image from text (custom sprite font)
void ImageFormat(Image *image, int newFormat);                                                     // Convert image data to desired format
void ImageToPOT(Image *image, Color fill);                                                         // Convert image to POT (power-of-two)
void ImageCrop(Image *image, Rectangle crop);                                                      // Crop an image to a defined rectangle
void ImageAlphaCrop(Image *image, float threshold);                                                // Crop image depending on alpha value
void ImageAlphaClear(Image *image, Color color, float threshold);                                  // Clear alpha channel to desired color
void ImageAlphaMask(Image *image, Image alphaMask);                                                // Apply alpha mask to image
void ImageAlphaPremultiply(Image *image);                                                          // Premultiply alpha channel
void ImageBlurGaussian(Image *image, int blurSize);                                                // Apply Gaussian blur using a box blur approximation
void ImageKernelConvolution(Image *image, const float *kernel, int kernelSize);                    // Apply custom square convolution kernel to image
void ImageResize(Image *image, int newWidth, int newHeight);                                       // Resize image (Bicubic scaling algorithm)
void ImageResizeNN(Image *image, int newWidth,int newHeight);                                      // Resize image (Nearest-Neighbor scaling algorithm)
void ImageResizeCanvas(Image *image, int newWidth, int newHeight, int offsetX, int offsetY, Color fill); // Resize canvas and fill with color
void ImageMipmaps(Image *image);                                                                   // Compute all mipmap levels for a provided image
void ImageDither(Image *image, int rBpp, int gBpp, int bBpp, int aBpp);                            // Dither image data to 16bpp or lower (Floyd-Steinberg dithering)
void ImageFlipVertical(Image *image);                                                              // Flip image vertically
void ImageFlipHorizontal(Image *image);                                                            // Flip image horizontally
void ImageRotate(Image *image, int degrees);                                                       // Rotate image by input angle in degrees (-359 to 359)
void ImageRotateCW(Image *image);                                                                  // Rotate image clockwise 90deg
void ImageRotateCCW(Image *image);                                                                 // Rotate image counter-clockwise 90deg
void ImageColorTint(Image *image, Color color);                                                    // Modify image color: tint
void ImageColorInvert(Image *image);                                                               // Modify image color: invert
void ImageColorGrayscale(Image *image);                                                            // Modify image color: grayscale
void ImageColorContrast(Image *image, float contrast);                                             // Modify image color: contrast (-100 to 100)
void ImageColorBrightness(Image *image, int brightness);                                           // Modify image color: brightness (-255 to 255)
void ImageColorReplace(Image *image, Color color, Color replace);                                  // Modify image color: replace color
Color *LoadImageColors(Image image);                                                               // Load color data from image as a Color array (RGBA - 32bit)
Color *LoadImagePalette(Image image, int maxPaletteSize, int *colorCount);                         // Load colors palette from image as a Color array (RGBA - 32bit)
void UnloadImageColors(Color *colors);                                                             // Unload color data loaded with LoadImageColors()
void UnloadImagePalette(Color *colors);                                                            // Unload colors palette loaded with LoadImagePalette()
Rectangle GetImageAlphaBorder(Image image, float threshold);                                       // Get image alpha border rectangle
Color GetImageColor(Image image, int x, int y);                                                    // Get image pixel color at (x, y) position

// Image drawing functions
// NOTE: Image software-rendering functions (CPU)
void ImageClearBackground(Image *dst, Color color);                                                // Clear image background with given color
void ImageDrawPixel(Image *dst, int posX, int posY, Color color);                                  // Draw pixel within an image
void ImageDrawPixelV(Image *dst, Vector2 position, Color color);                                   // Draw pixel within an image (Vector version)
void ImageDrawLine(Image *dst, int startPosX, int startPosY, int endPosX, int endPosY, Color color); // Draw line within an image
void ImageDrawLineV(Image *dst, Vector2 start, Vector2 end, Color color);                          // Draw line within an image (Vector version)
void ImageDrawLineEx(Image *dst, Vector2 start, Vector2 end, int thick, Color color);              // Draw a line defining thickness within an image
void ImageDrawCircle(Image *dst, int centerX, int centerY, int radius, Color color);               // Draw a filled circle within an image
void ImageDrawCircleV(Image *dst, Vector2 center, int radius, Color color);                        // Draw a filled circle within an image (Vector version)
void ImageDrawCircleLines(Image *dst, int centerX, int centerY, int radius, Color color);          // Draw circle outline within an image
void ImageDrawCircleLinesV(Image *dst, Vector2 center, int radius, Color color);                   // Draw circle outline within an image (Vector version)
void ImageDrawRectangle(Image *dst, int posX, int posY, int width, int height, Color color);       // Draw rectangle within an image
void ImageDrawRectangleV(Image *dst, Vector2 position, Vector2 size, Color color);                 // Draw rectangle within an image (Vector version)
void ImageDrawRectangleRec(Image *dst, Rectangle rec, Color color);                                // Draw rectangle within an image
void ImageDrawRectangleLines(Image *dst, Rectangle rec, int thick, Color color);                   // Draw rectangle lines within an image
void ImageDrawTriangle(Image *dst, Vector2 v1, Vector2 v2, Vector2 v3, Color color);               // Draw triangle within an image
void ImageDrawTriangleEx(Image *dst, Vector2 v1, Vector2 v2, Vector2 v3, Color c1, Color c2, Color c3); // Draw triangle with interpolated colors within an image
void ImageDrawTriangleLines(Image *dst, Vector2 v1, Vector2 v2, Vector2 v3, Color color);          // Draw triangle outline within an image
void ImageDrawTriangleFan(Image *dst, Vector2 *points, int pointCount, Color color);               // Draw a triangle fan defined by points within an image (first vertex is the center)
void ImageDrawTriangleStrip(Image *dst, Vector2 *points, int pointCount, Color color);             // Draw a triangle strip defined by points within an image
void ImageDraw(Image *dst, Image src, Rectangle srcRec, Rectangle dstRec, Color tint);             // Draw a source image within a destination image (tint applied to source)
void ImageDrawText(Image *dst, const char *text, int posX, int posY, int fontSize, Color color);   // Draw text (using default font) within an image (destination)
void ImageDrawTextEx(Image *dst, Font font, const char *text, Vector2 position, float fontSize, float spacing, Color tint); // Draw text (custom sprite font) within an image (destination)

// Texture loading functions
// NOTE: These functions require GPU access
Texture2D LoadTexture(const char *fileName);                                                       // Load texture from file into GPU memory (VRAM)
Texture2D LoadTextureFromImage(Image image);                                                       // Load texture from image data
TextureCubemap LoadTextureCubemap(Image image, int layout);                                        // Load cubemap from image, multiple image cubemap layouts supported
RenderTexture2D LoadRenderTexture(int width, int height);                                          // Load texture for rendering (framebuffer)
bool IsTextureValid(Texture2D texture);                                                            // Check if a texture is valid (loaded in GPU)
void UnloadTexture(Texture2D texture);                                                             // Unload texture from GPU memory (VRAM)
bool IsRenderTextureValid(RenderTexture2D target);                                                 // Check if a render texture is valid (loaded in GPU)
void UnloadRenderTexture(RenderTexture2D target);                                                  // Unload render texture from GPU memory (VRAM)
void UpdateTexture(Texture2D texture, const void *pixels);                                         // Update GPU texture with new data
void UpdateTextureRec(Texture2D texture, Rectangle rec, const void *pixels);                       // Update GPU texture rectangle with new data

// Texture configuration functions
void GenTextureMipmaps(Texture2D *texture);                                                        // Generate GPU mipmaps for a texture
void SetTextureFilter(Texture2D texture, int filter);                                              // Set texture scaling filter mode
void SetTextureWrap(Texture2D texture, int wrap);                                                  // Set texture wrapping mode

// Texture drawing functions
void DrawTexture(Texture2D texture, int posX, int posY, Color tint);                               // Draw a Texture2D
void DrawTextureV(Texture2D texture, Vector2 position, Color tint);                                // Draw a Texture2D with position defined as Vector2
void DrawTextureEx(Texture2D texture, Vector2 position, float rotation, float scale, Color tint);  // Draw a Texture2D with extended parameters
void DrawTextureRec(Texture2D texture, Rectangle source, Vector2 position, Color tint);            // Draw a part of a texture defined by a rectangle
void DrawTexturePro(Texture2D texture, Rectangle source, Rectangle dest, Vector2 origin, float rotation, Color tint); // Draw a part of a texture defined by a rectangle with 'pro' parameters
void DrawTextureNPatch(Texture2D texture, NPatchInfo nPatchInfo, Rectangle dest, Vector2 origin, float rotation, Color tint); // Draws a texture (or part of it) that stretches or shrinks nicely

// Color/pixel related functions
bool ColorIsEqual(Color col1, Color col2);                            // Check if two colors are equal
Color Fade(Color color, float alpha);                                 // Get color with alpha applied, alpha goes from 0.0f to 1.0f
int ColorToInt(Color color);                                          // Get hexadecimal value for a Color (0xRRGGBBAA)
Vector4 ColorNormalize(Color color);                                  // Get Color normalized as float [0..1]
Color ColorFromNormalized(Vector4 normalized);                        // Get Color from normalized values [0..1]
Vector3 ColorToHSV(Color color);                                      // Get HSV values for a Color, hue [0..360], saturation/value [0..1]
Color ColorFromHSV(float hue, float saturation, float value);         // Get a Color from HSV values, hue [0..360], saturation/value [0..1]
Color ColorTint(Color color, Color tint);                             // Get color multiplied with another color
Color ColorBrightness(Color color, float factor);                     // Get color with brightness correction, brightness factor goes from -1.0f to 1.0f
Color ColorContrast(Color color, float contrast);                     // Get color with contrast correction, contrast values between -1.0f and 1.0f
Color ColorAlpha(Color color, float alpha);                           // Get color with alpha applied, alpha goes from 0.0f to 1.0f
Color ColorAlphaBlend(Color dst, Color src, Color tint);              // Get src alpha-blended into dst color with tint
Color ColorLerp(Color color1, Color color2, float factor);            // Get color lerp interpolation between two colors, factor [0.0f..1.0f]
Color GetColor(unsigned int hexValue);                                // Get Color structure from hexadecimal value
Color GetPixelColor(void *srcPtr, int format);                        // Get Color from a source pixel pointer of certain format
void SetPixelColor(void *dstPtr, Color color, int format);            // Set color formatted into destination pixel pointer
int GetPixelDataSize(int width, int height, int format);              // Get pixel data size in bytes for certain format

//------------------------------------------------------------------------------------
// Font Loading and Text Drawing Functions (Module: text)
//------------------------------------------------------------------------------------

// Font loading/unloading functions
Font GetFontDefault(void);                                                            // Get the default Font
Font LoadFont(const char *fileName);                                                  // Load font from file into GPU memory (VRAM)
Font LoadFontEx(const char *fileName, int fontSize, int *codepoints, int codepointCount); // Load font from file with extended parameters, use NULL for codepoints and 0 for codepointCount to load the default character set, font size is provided in pixels height
Font LoadFontFromImage(Image image, Color key, int firstChar);                        // Load font from Image (XNA style)
Font LoadFontFromMemory(const char *fileType, const unsigned char *fileData, int dataSize, int fontSize, int *codepoints, int codepointCount); // Load font from memory buffer, fileType refers to extension: i.e. '.ttf'
bool IsFontValid(Font font);                                                          // Check if a font is valid (font data loaded, WARNING: GPU texture not checked)
GlyphInfo *LoadFontData(const unsigned char *fileData, int dataSize, int fontSize, int *codepoints, int codepointCount, int type); // Load font data for further use
Image GenImageFontAtlas(const GlyphInfo *glyphs, Rectangle **glyphRecs, int glyphCount, int fontSize, int padding, int packMethod); // Generate image font atlas using chars info
void UnloadFontData(GlyphInfo *glyphs, int glyphCount);                               // Unload font chars info data (RAM)
void UnloadFont(Font font);                                                           // Unload font from GPU memory (VRAM)
bool ExportFontAsCode(Font font, const char *fileName);                               // Export font as code file, returns true on success

// Text drawing functions
void DrawFPS(int posX, int posY);                                                     // Draw current FPS
void DrawText(const char *text, int posX, int posY, int fontSize, Color color);       // Draw text (using default font)
void DrawTextEx(Font font, const char *text, Vector2 position, float fontSize, float spacing, Color tint); // Draw text using font and additional parameters
void DrawTextPro(Font font, const char *text, Vector2 position, Vector2 origin, float rotation, float fontSize, float spacing, Color tint); // Draw text using Font and pro parameters (rotation)
void DrawTextCodepoint(Font font, int codepoint, Vector2 position, float fontSize, Color tint); // Draw one character (codepoint)
void DrawTextCodepoints(Font font, const int *codepoints, int codepointCount, Vector2 position, float fontSize, float spacing, Color tint); // Draw multiple character (codepoint)

// Text font info functions
void SetTextLineSpacing(int spacing);                                                 // Set vertical line spacing when drawing with line-breaks
int MeasureText(const char *text, int fontSize);                                      // Measure string width for default font
Vector2 MeasureTextEx(Font font, const char *text, float fontSize, float spacing);    // Measure string size for Font
int GetGlyphIndex(Font font, int codepoint);                                          // Get glyph index position in font for a codepoint (unicode character), fallback to '?' if not found
GlyphInfo GetGlyphInfo(Font font, int codepoint);                                     // Get glyph font info data for a codepoint (unicode character), fallback to '?' if not found
Rectangle GetGlyphAtlasRec(Font font, int codepoint);                                 // Get glyph rectangle in font atlas for a codepoint (unicode character), fallback to '?' if not found

// Text codepoints management functions (unicode characters)
char *LoadUTF8(const int *codepoints, int length);                // Load UTF-8 text encoded from codepoints array
void UnloadUTF8(char *text);                                      // Unload UTF-8 text encoded from codepoints array
int *LoadCodepoints(const char *text, int *count);                // Load all codepoints from a UTF-8 text string, codepoints count returned by parameter
void UnloadCodepoints(int *codepoints);                           // Unload codepoints data from memory
int GetCodepointCount(const char *text);                          // Get total number of codepoints in a UTF-8 encoded string
int GetCodepoint(const char *text, int *codepointSize);           // Get next codepoint in a UTF-8 encoded string, 0x3f('?') is returned on failure
int GetCodepointNext(const char *text, int *codepointSize);       // Get next codepoint in a UTF-8 encoded string, 0x3f('?') is returned on failure
int GetCodepointPrevious(const char *text, int *codepointSize);   // Get previous codepoint in a UTF-8 encoded string, 0x3f('?') is returned on failure
const char *CodepointToUTF8(int codepoint, int *utf8Size);        // Encode one codepoint into UTF-8 byte array (array length returned as parameter)

// Text strings management functions (no UTF-8 strings, only byte chars)
// NOTE: Some strings allocate memory internally for returned strings, just be careful!
int TextCopy(char *dst, const char *src);                                             // Copy one string to another, returns bytes copied
bool TextIsEqual(const char *text1, const char *text2);                               // Check if two text string are equal
unsigned int TextLength(const char *text);                                            // Get text length, checks for '\0' ending
const char *TextFormat(const char *text, ...);                                        // Text formatting with variables (sprintf() style)
const char *TextSubtext(const char *text, int position, int length);                  // Get a piece of a text string
char *TextReplace(const char *text, const char *replace, const char *by);             // Replace text string (WARNING: memory must be freed!)
char *TextInsert(const char *text, const char *insert, int position);                 // Insert text in a position (WARNING: memory must be freed!)
const char *TextJoin(const char **textList, int count, const char *delimiter);        // Join text strings with delimiter
const char **TextSplit(const char *text, char delimiter, int *count);                 // Split text into multiple strings
void TextAppend(char *text, const char *append, int *position);                       // Append text at specific position and move cursor!
int TextFindIndex(const char *text, const char *find);                                // Find first text occurrence within a string
const char *TextToUpper(const char *text);                      // Get upper case version of provided string
const char *TextToLower(const char *text);                      // Get lower case version of provided string
const char *TextToPascal(const char *text);                     // Get Pascal case notation version of provided string
const char *TextToSnake(const char *text);                      // Get Snake case notation version of provided string
const char *TextToCamel(const char *text);                      // Get Camel case notation version of provided string

int TextToInteger(const char *text);                            // Get integer value from text (negative values not supported)
float TextToFloat(const char *text);                            // Get float value from text (negative values not supported)

//------------------------------------------------------------------------------------
// Basic 3d Shapes Drawing Functions (Module: models)
//------------------------------------------------------------------------------------

// Basic geometric 3D shapes drawing functions
void DrawLine3D(Vector3 startPos, Vector3 endPos, Color color);                                    // Draw a line in 3D world space
void DrawPoint3D(Vector3 position, Color color);                                                   // Draw a point in 3D space, actually a small line
void DrawCircle3D(Vector3 center, float radius, Vector3 rotationAxis, float rotationAngle, Color color); // Draw a circle in 3D world space
void DrawTriangle3D(Vector3 v1, Vector3 v2, Vector3 v3, Color color);                              // Draw a color-filled triangle (vertex in counter-clockwise order!)
void DrawTriangleStrip3D(const Vector3 *points, int pointCount, Color color);                      // Draw a triangle strip defined by points
void DrawCube(Vector3 position, float width, float height, float length, Color color);             // Draw cube
void DrawCubeV(Vector3 position, Vector3 size, Color color);                                       // Draw cube (Vector version)
void DrawCubeWires(Vector3 position, float width, float height, float length, Color color);        // Draw cube wires
void DrawCubeWiresV(Vector3 position, Vector3 size, Color color);                                  // Draw cube wires (Vector version)
void DrawSphere(Vector3 centerPos, float radius, Color color);                                     // Draw sphere
void DrawSphereEx(Vector3 centerPos, float radius, int rings, int slices, Color color);            // Draw sphere with extended parameters
void DrawSphereWires(Vector3 centerPos, float radius, int rings, int slices, Color color);         // Draw sphere wires
void DrawCylinder(Vector3 position, float radiusTop, float radiusBottom, float height, int slices, Color color); // Draw a cylinder/cone
void DrawCylinderEx(Vector3 startPos, Vector3 endPos, float startRadius, float endRadius, int sides, Color color); // Draw a cylinder with base at startPos and top at endPos
void DrawCylinderWires(Vector3 position, float radiusTop, float radiusBottom, float height, int slices, Color color); // Draw a cylinder/cone wires
void DrawCylinderWiresEx(Vector3 startPos, Vector3 endPos, float startRadius, float endRadius, int sides, Color color); // Draw a cylinder wires with base at startPos and top at endPos
void DrawCapsule(Vector3 startPos, Vector3 endPos, float radius, int slices, int rings, Color color); // Draw a capsule with the center of its sphere caps at startPos and endPos
void DrawCapsuleWires(Vector3 startPos, Vector3 endPos, float radius, int slices, int rings, Color color); // Draw capsule wireframe with the center of its sphere caps at startPos and endPos
void DrawPlane(Vector3 centerPos, Vector2 size, Color color);                                      // Draw a plane XZ
void DrawRay(Ray ray, Color color);                                                                // Draw a ray line
void DrawGrid(int slices, float spacing);                                                          // Draw a grid (centered at (0, 0, 0))

//------------------------------------------------------------------------------------
// Model 3d Loading and Drawing Functions (Module: models)
//------------------------------------------------------------------------------------

// Model management functions
Model LoadModel(const char *fileName);                                                // Load model from files (meshes and materials)
Model LoadModelFromMesh(Mesh mesh);                                                   // Load model from generated mesh (default material)
bool IsModelValid(Model model);                                                       // Check if a model is valid (loaded in GPU, VAO/VBOs)
void UnloadModel(Model model);                                                        // Unload model (including meshes) from memory (RAM and/or VRAM)
BoundingBox GetModelBoundingBox(Model model);                                         // Compute model bounding box limits (considers all meshes)

// Model drawing functions
void DrawModel(Model model, Vector3 position, float scale, Color tint);               // Draw a model (with texture if set)
void DrawModelEx(Model model, Vector3 position, Vector3 rotationAxis, float rotationAngle, Vector3 scale, Color tint); // Draw a model with extended parameters
void DrawModelWires(Model model, Vector3 position, float scale, Color tint);          // Draw a model wires (with texture if set)
void DrawModelWiresEx(Model model, Vector3 position, Vector3 rotationAxis, float rotationAngle, Vector3 scale, Color tint); // Draw a model wires (with texture if set) with extended parameters
void DrawModelPoints(Model model, Vector3 position, float scale, Color tint); // Draw a model as points
void DrawModelPointsEx(Model model, Vector3 position, Vector3 rotationAxis, float rotationAngle, Vector3 scale, Color tint); // Draw a model as points with extended parameters
void DrawBoundingBox(BoundingBox box, Color color);                                   // Draw bounding box (wires)
void DrawBillboard(Camera camera, Texture2D texture, Vector3 position, float scale, Color tint);   // Draw a billboard texture
void DrawBillboardRec(Camera camera, Texture2D texture, Rectangle source, Vector3 position, Vector2 size, Color tint); // Draw a billboard texture defined by source
void DrawBillboardPro(Camera camera, Texture2D texture, Rectangle source, Vector3 position, Vector3 up, Vector2 size, Vector2 origin, float rotation, Color tint); // Draw a billboard texture defined by source and rotation

// Mesh management functions
void UploadMesh(Mesh *mesh, bool dynamic);                                            // Upload mesh vertex data in GPU and provide VAO/VBO ids
void UpdateMeshBuffer(Mesh mesh, int index, const void *data, int dataSize, int offset); // Update mesh vertex data in GPU for a specific buffer index
void UnloadMesh(Mesh mesh);                                                           // Unload mesh data from CPU and GPU
void DrawMesh(Mesh mesh, Material material, Matrix transform);                        // Draw a 3d mesh with material and transform
void DrawMeshInstanced(Mesh mesh, Material material, const Matrix *transforms, int instances); // Draw multiple mesh instances with material and different transforms
BoundingBox GetMeshBoundingBox(Mesh mesh);                                            // Compute mesh bounding box limits
void GenMeshTangents(Mesh *mesh);                                                     // Compute mesh tangents
bool ExportMesh(Mesh mesh, const char *fileName);                                     // Export mesh data to file, returns true on success
bool ExportMeshAsCode(Mesh mesh, const char *fileName);                               // Export mesh as code file (.h) defining multiple arrays of vertex attributes

// Mesh generation functions
Mesh GenMeshPoly(int sides, float radius);                                            // Generate polygonal mesh
Mesh GenMeshPlane(float width, float length, int resX, int resZ);                     // Generate plane mesh (with subdivisions)
Mesh GenMeshCube(float width, float height, float length);                            // Generate cuboid mesh
Mesh GenMeshSphere(float radius, int rings, int slices);                              // Generate sphere mesh (standard sphere)
Mesh GenMeshHemiSphere(float radius, int rings, int slices);                          // Generate half-sphere mesh (no bottom cap)
Mesh GenMeshCylinder(float radius, float height, int slices);                         // Generate cylinder mesh
Mesh GenMeshCone(float radius, float height, int slices);                             // Generate cone/pyramid mesh
Mesh GenMeshTorus(float radius, float size, int radSeg, int sides);                   // Generate torus mesh
Mesh GenMeshKnot(float radius, float size, int radSeg, int sides);                    // Generate trefoil knot mesh
Mesh GenMeshHeightmap(Image heightmap, Vector3 size);                                 // Generate heightmap mesh from image data
Mesh GenMeshCubicmap(Image cubicmap, Vector3 cubeSize);                               // Generate cubes-based map mesh from image data

// Material loading/unloading functions
Material *LoadMaterials(const char *fileName, int *materialCount);                    // Load materials from model file
Material LoadMaterialDefault(void);                                                   // Load default material (Supports: DIFFUSE, SPECULAR, NORMAL maps)
bool IsMaterialValid(Material material);                                              // Check if a material is valid (shader assigned, map textures loaded in GPU)
void UnloadMaterial(Material material);                                               // Unload material from GPU memory (VRAM)
void SetMaterialTexture(Material *material, int mapType, Texture2D texture);          // Set texture for a material map type (MATERIAL_MAP_DIFFUSE, MATERIAL_MAP_SPECULAR...)
void SetModelMeshMaterial(Model *model, int meshId, int materialId);                  // Set material for a mesh

// Model animations loading/unloading functions
ModelAnimation *LoadModelAnimations(const char *fileName, int *animCount);            // Load model animations from file
void UpdateModelAnimation(Model model, ModelAnimation anim, int frame);               // Update model animation pose (CPU)
void UpdateModelAnimationBones(Model model, ModelAnimation anim, int frame);          // Update model animation mesh bone matrices (GPU skinning)
void UnloadModelAnimation(ModelAnimation anim);                                       // Unload animation data
void UnloadModelAnimations(ModelAnimation *animations, int animCount);                // Unload animation array data
bool IsModelAnimationValid(Model model, ModelAnimation anim);                         // Check model animation skeleton match

// Collision detection functions
bool CheckCollisionSpheres(Vector3 center1, float radius1, Vector3 center2, float radius2);   // Check collision between two spheres
bool CheckCollisionBoxes(BoundingBox box1, BoundingBox box2);                                 // Check collision between two bounding boxes
bool CheckCollisionBoxSphere(BoundingBox box, Vector3 center, float radius);                  // Check collision between box and sphere
RayCollision GetRayCollisionSphere(Ray ray, Vector3 center, float radius);                    // Get collision info between ray and sphere
RayCollision GetRayCollisionBox(Ray ray, BoundingBox box);                                    // Get collision info between ray and box
RayCollision GetRayCollisionMesh(Ray ray, Mesh mesh, Matrix transform);                       // Get collision info between ray and mesh
RayCollision GetRayCollisionTriangle(Ray ray, Vector3 p1, Vector3 p2, Vector3 p3);            // Get collision info between ray and triangle
RayCollision GetRayCollisionQuad(Ray ray, Vector3 p1, Vector3 p2, Vector3 p3, Vector3 p4);    // Get collision info between ray and quad

//------------------------------------------------------------------------------------
// Audio Loading and Playing Functions (Module: audio)
//------------------------------------------------------------------------------------
typedef void (*AudioCallback)(void *bufferData, unsigned int frames);

// Audio device management functions
void InitAudioDevice(void);                                     // Initialize audio device and context
void CloseAudioDevice(void);                                    // Close the audio device and context
bool IsAudioDeviceReady(void);                                  // Check if audio device has been initialized successfully
void SetMasterVolume(float volume);                             // Set master volume (listener)
float GetMasterVolume(void);                                    // Get master volume (listener)

// Wave/Sound loading/unloading functions
Wave LoadWave(const char *fileName);                            // Load wave data from file
Wave LoadWaveFromMemory(const char *fileType, const unsigned char *fileData, int dataSize); // Load wave from memory buffer, fileType refers to extension: i.e. '.wav'
bool IsWaveValid(Wave wave);                                    // Checks if wave data is valid (data loaded and parameters)
Sound LoadSound(const char *fileName);                          // Load sound from file
Sound LoadSoundFromWave(Wave wave);                             // Load sound from wave data
Sound LoadSoundAlias(Sound source);                             // Create a new sound that shares the same sample data as the source sound, does not own the sound data
bool IsSoundValid(Sound sound);                                 // Checks if a sound is valid (data loaded and buffers initialized)
void UpdateSound(Sound sound, const void *data, int sampleCount); // Update sound buffer with new data
void UnloadWave(Wave wave);                                     // Unload wave data
void UnloadSound(Sound sound);                                  // Unload sound
void UnloadSoundAlias(Sound alias);                             // Unload a sound alias (does not deallocate sample data)
bool ExportWave(Wave wave, const char *fileName);               // Export wave data to file, returns true on success
bool ExportWaveAsCode(Wave wave, const char *fileName);         // Export wave sample data to code (.h), returns true on success

// Wave/Sound management functions
void PlaySound(Sound sound);                                    // Play a sound
void StopSound(Sound sound);                                    // Stop playing a sound
void PauseSound(Sound sound);                                   // Pause a sound
void ResumeSound(Sound sound);                                  // Resume a paused sound
bool IsSoundPlaying(Sound sound);                               // Check if a sound is currently playing
void SetSoundVolume(Sound sound, float volume);                 // Set volume for a sound (1.0 is max level)
void SetSoundPitch(Sound sound, float pitch);                   // Set pitch for a sound (1.0 is base level)
void SetSoundPan(Sound sound, float pan);                       // Set pan for a sound (0.5 is center)
Wave WaveCopy(Wave wave);                                       // Copy a wave to a new wave
void WaveCrop(Wave *wave, int initFrame, int finalFrame);       // Crop a wave to defined frames range
void WaveFormat(Wave *wave, int sampleRate, int sampleSize, int channels); // Convert wave data to desired format
float *LoadWaveSamples(Wave wave);                              // Load samples data from wave as a 32bit float data array
void UnloadWaveSamples(float *samples);                         // Unload samples data loaded with LoadWaveSamples()

// Music management functions
Music LoadMusicStream(const char *fileName);                    // Load music stream from file
Music LoadMusicStreamFromMemory(const char *fileType, const unsigned char *data, int dataSize); // Load music stream from data
bool IsMusicValid(Music music);                                 // Checks if a music stream is valid (context and buffers initialized)
void UnloadMusicStream(Music music);                            // Unload music stream
void PlayMusicStream(Music music);                              // Start music playing
bool IsMusicStreamPlaying(Music music);                         // Check if music is playing
void UpdateMusicStream(Music music);                            // Updates buffers for music streaming
void StopMusicStream(Music music);                              // Stop music playing
void PauseMusicStream(Music music);                             // Pause music playing
void ResumeMusicStream(Music music);                            // Resume playing paused music
void SeekMusicStream(Music music, float position);              // Seek music to a position (in seconds)
void SetMusicVolume(Music music, float volume);                 // Set volume for music (1.0 is max level)
void SetMusicPitch(Music music, float pitch);                   // Set pitch for a music (1.0 is base level)
void SetMusicPan(Music music, float pan);                       // Set pan for a music (0.5 is center)
float GetMusicTimeLength(Music music);                          // Get music time length (in seconds)
float GetMusicTimePlayed(Music music);                          // Get current music time played (in seconds)

// AudioStream management functions
AudioStream LoadAudioStream(unsigned int sampleRate, unsigned int sampleSize, unsigned int channels); // Load audio stream (to stream raw audio pcm data)
bool IsAudioStreamValid(AudioStream stream);                    // Checks if an audio stream is valid (buffers initialized)
void UnloadAudioStream(AudioStream stream);                     // Unload audio stream and free memory
void UpdateAudioStream(AudioStream stream, const void *data, int frameCount); // Update audio stream buffers with data
bool IsAudioStreamProcessed(AudioStream stream);                // Check if any audio stream buffers requires refill
void PlayAudioStream(AudioStream stream);                       // Play audio stream
void PauseAudioStream(AudioStream stream);                      // Pause audio stream
void ResumeAudioStream(AudioStream stream);                     // Resume audio stream
bool IsAudioStreamPlaying(AudioStream stream);                  // Check if audio stream is playing
void StopAudioStream(AudioStream stream);                       // Stop audio stream
void SetAudioStreamVolume(AudioStream stream, float volume);    // Set volume for audio stream (1.0 is max level)
void SetAudioStreamPitch(AudioStream stream, float pitch);      // Set pitch for audio stream (1.0 is base level)
void SetAudioStreamPan(AudioStream stream, float pan);          // Set pan for audio stream (0.5 is centered)
void SetAudioStreamBufferSizeDefault(int size);                 // Default size for new audio streams
void SetAudioStreamCallback(AudioStream stream, AudioCallback callback); // Audio thread callback to request new data

void AttachAudioStreamProcessor(AudioStream stream, AudioCallback processor); // Attach audio stream processor to stream, receives the samples as 'float'
void DetachAudioStreamProcessor(AudioStream stream, AudioCallback processor); // Detach audio stream processor from stream

void AttachAudioMixedProcessor(AudioCallback processor); // Attach audio stream processor to the entire audio pipeline, receives the samples as 'float'
void DetachAudioMixedProcessor(AudioCallback processor); // Detach audio stream processor from the entire audio pipeline

]]


-- colors
local function new_color(r, g, b, a)
  return ffi.new("Color", r, g, b, a)
end

rl.Rectangle=function(x,y,width,height)
    return ffi.new("Rectangle",x,y,width,height)
end
rl.toCRectangle=function(t)
    local rect=ffi.typeof("Rectangle")
    if type(t)=="table" then
        return ffi.new("Rectangle",t.x,t.y,t.width,t.height)
    elseif ffi.istype(rect,t) then
        return t
    end
    error("This is not a rectangle!")
end

rl.Color=function(r,g,b,a)
    return ffi.new("Color",r,g,b,a)
end
rl.toCColor=function(t)
    local color=ffi.typeof("Color")
    if type(t)=="table" then
        return ffi.new("Color",t.r,t.g,t.b,t.a)
    elseif ffi.istype(color,t)then
        return t
    end
    error("This is not a Color!")
end

rl.Vector2=function(x,y)
    return ffi.new("Vector2",x,y)
end
rl.Vector3=function(x,y,z)
    return ffi.new("Vector3",x,y,z)
end
rl.Vector4=function(x,y,z,w)
    return ffi.new("Vector4",x,y,z,w)
end
rl.Camera2D=function(offset,target,rotation,zoom)
    return ffi.new("Camera2D",offset,target,rotation,zoom)
end
rl.Camera3D=function(position,target,up,fovy,projection)
    return ffi.new("Camera3D",position,target,up,fovy,projection)
end


rl.LIGHTGRAY = new_color(200, 200, 200, 255)
rl.GRAY = new_color(130, 130, 130, 255)
rl.DARKGRAY = new_color(80, 80, 80, 255)
rl.YELLOW = new_color(253, 249, 0, 255)
rl.GOLD = new_color(255, 203, 0, 255)
rl.ORANGE = new_color(255, 161, 0, 255)
rl.PINK = new_color(255, 109, 194, 255)
rl.RED = new_color(230, 41, 55, 255)
rl.MAROON = new_color(190, 33, 55, 255)
rl.GREEN = new_color(0, 228, 48, 255)
rl.LIME = new_color(0, 158, 47, 255)
rl.DARKGREEN = new_color(0, 117, 44, 255)
rl.SKYBLUE = new_color(102, 191, 255, 255)
rl.BLUE = new_color(0, 121, 241, 255)
rl.DARKBLUE = new_color(0, 82, 172, 255)
rl.PURPLE = new_color(200, 122, 255, 255)
rl.VIOLET = new_color(135, 60, 190, 255)
rl.DARKPURPLE = new_color(112, 31, 126, 255)
rl.BEIGE = new_color(211, 176, 131, 255)
rl.BROWN = new_color(127, 106, 79, 255)
rl.DARKBROWN = new_color(76, 63, 47, 255)
rl.WHITE = new_color(255, 255, 255, 255)
rl.BLACK = new_color(0, 0, 0, 255)
rl.BLANK = new_color(0, 0, 0, 0)
rl.MAGENTA = new_color(255, 0, 255, 255)
rl.RAYWHITE = new_color(245, 245, 245, 255)

rl.SHADER_LOC_MAP_DIFFUSE = C.SHADER_LOC_MAP_ALBEDO
rl.SHADER_LOC_MAP_SPECULAR = C.SHADER_LOC_MAP_METALNESS
rl.MATERIAL_MAP_DIFFUSE = C.MATERIAL_MAP_ALBEDO
rl.MATERIAL_MAP_SPECULAR = C.MATERIAL_MAP_METALNESS


function rl.ref(obj)
    return ffi.cast(ffi.typeof("$ *", obj), obj)
end

local callback_keepalive = {
    trace = {},
    load_file_data = {},
    save_file_data = {},
    load_file_text = {},
    save_file_text = {},
    audio_stream = {},
    audio_stream_processors = {},
    audio_mixed_processors = {},
}

local function stream_key(stream)
    if stream ~= nil and stream.buffer ~= nil then
        return stream.buffer
    end
    return stream
end

local function keep_single(slot, cb_type, cb)
    if cb == nil then
        slot.func = nil
        slot.cb = nil
        return nil
    end
    if type(cb) == "function" then
        if slot.func == cb and slot.cb ~= nil then
            return slot.cb
        end
        slot.func = cb
        slot.cb = ffi.cast(cb_type, cb)
        return slot.cb
    end
    slot.func = nil
    slot.cb = ffi.cast(cb_type, cb)
    return slot.cb
end

local function keep_multi(map, cb_type, cb)
    if cb == nil then
        error("callback is required", 2)
    end
    local key = cb
    local entry = map[key]
    if entry ~= nil then
        return entry
    end
    local casted = (type(cb) == "function") and ffi.cast(cb_type, cb) or ffi.cast(cb_type, cb)
    map[key] = casted
    return casted
end

local function drop_multi(map, cb)
    if cb == nil then
        error("callback is required", 2)
    end
    local entry = map[cb]
    if entry ~= nil then
        map[cb] = nil
        return entry
    end
    for k, v in pairs(map) do
        if v == cb then
            map[k] = nil
            return v
        end
    end
    return cb
end

local exports={
    "InitWindow",
    "WindowShouldClose",
    "BeginDrawing",
    "ClearBackground",
    "DrawText",
    "EndDrawing",
    "CloseWindow",
    "SetConfigFlags",
    "GetRandomValue",
    "SetTargetFPS",
    "IsKeyDown",
    "IsKeyPressed",
    "GetMouseWheelMove",
    "BeginMode2D",
    "BeginMode3D",
    "EndMode2D",
    "EndMode3D",
    "DrawRectangle",
    "DrawRectangleRec",
    "DrawCube",
    "DrawCubeWires",
    "DrawGrid",
    "DrawLine",
    "Fade",
    "DrawRectangleLines",
    "UpdateCamera",
    "DisableCursor",
}
package.cpath = './build/subprojects/raylib/?.dylib;'..'./build/subprojects/raylib/?.so;' .. package.cpath
local c_raylib = ffi.load('libraylib')
for _,name in ipairs(exports) do
    rl[name]=c_raylib[name]
end

local TraceLogCallback = ffi.typeof("TraceLogCallback")
local LoadFileDataCallback = ffi.typeof("LoadFileDataCallback")
local SaveFileDataCallback = ffi.typeof("SaveFileDataCallback")
local LoadFileTextCallback = ffi.typeof("LoadFileTextCallback")
local SaveFileTextCallback = ffi.typeof("SaveFileTextCallback")
local AudioCallback = ffi.typeof("AudioCallback")

rl.SetTraceLogCallback = function(cb)
    c_raylib.SetTraceLogCallback(keep_single(callback_keepalive.trace, TraceLogCallback, cb))
end

rl.SetLoadFileDataCallback = function(cb)
    c_raylib.SetLoadFileDataCallback(keep_single(callback_keepalive.load_file_data, LoadFileDataCallback, cb))
end

rl.SetSaveFileDataCallback = function(cb)
    c_raylib.SetSaveFileDataCallback(keep_single(callback_keepalive.save_file_data, SaveFileDataCallback, cb))
end

rl.SetLoadFileTextCallback = function(cb)
    c_raylib.SetLoadFileTextCallback(keep_single(callback_keepalive.load_file_text, LoadFileTextCallback, cb))
end

rl.SetSaveFileTextCallback = function(cb)
    c_raylib.SetSaveFileTextCallback(keep_single(callback_keepalive.save_file_text, SaveFileTextCallback, cb))
end

rl.SetAudioStreamCallback = function(stream, cb)
    local key = stream_key(stream)
    local slot = callback_keepalive.audio_stream[key]
    if slot == nil then
        slot = {}
        callback_keepalive.audio_stream[key] = slot
    end
    c_raylib.SetAudioStreamCallback(stream, keep_single(slot, AudioCallback, cb))
end

rl.AttachAudioStreamProcessor = function(stream, cb)
    local key = stream_key(stream)
    local map = callback_keepalive.audio_stream_processors[key]
    if map == nil then
        map = {}
        callback_keepalive.audio_stream_processors[key] = map
    end
    c_raylib.AttachAudioStreamProcessor(stream, keep_multi(map, AudioCallback, cb))
end

rl.DetachAudioStreamProcessor = function(stream, cb)
    local key = stream_key(stream)
    local map = callback_keepalive.audio_stream_processors[key]
    if map == nil then
        map = {}
        callback_keepalive.audio_stream_processors[key] = map
    end
    c_raylib.DetachAudioStreamProcessor(stream, drop_multi(map, cb))
end

rl.AttachAudioMixedProcessor = function(cb)
    c_raylib.AttachAudioMixedProcessor(keep_multi(callback_keepalive.audio_mixed_processors, AudioCallback, cb))
end

rl.DetachAudioMixedProcessor = function(cb)
    c_raylib.DetachAudioMixedProcessor(drop_multi(callback_keepalive.audio_mixed_processors, cb))
end

rl.new = ffi.new

rl.__index = function (self, key)
    return C[key]
end

rl.__newindex = function ()
    error "rl table is readonly"
end

return rl
