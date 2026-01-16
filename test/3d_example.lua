package.path="./raylib/?.lua;"..package.path
local rl=require 'raylib'
local ffi=require 'ffi'


local screenWidth=800
local screenHeight=450

rl.InitWindow(screenWidth,screenHeight,"a 3d example")

local position=rl.Vector3(10.0,10.0,10.0)
local target=rl.Vector3(0.0,0.0,0.0)
local up=rl.Vector3(0.0,1.0,0.0)
local camera=ffi.new("Camera3D[1]", rl.Camera3D(position, target, up, 45.0, rl.CAMERA_PERSPECTIVE))
local cubePosition=rl.Vector3(0.0,0.0,0.0)
rl.DisableCursor()
rl.SetTargetFPS(60)
while not rl.WindowShouldClose() do
    rl.UpdateCamera(camera,rl.CAMERA_FREE)
    if (rl.IsKeyPressed(rl.KEY_Z)) then
        camera[0].target=rl.Vector3(0,0,0)
    end
    rl.BeginDrawing()
    rl.ClearBackground(rl.RAYWHITE)
    rl.BeginMode3D(camera[0])
    rl.DrawCube(cubePosition,2.0,2.0,2.0,rl.RED)
    rl.DrawCubeWires(cubePosition,2.0,2.0,2.0,rl.MAROON)
    rl.DrawGrid(10,1.0)
    rl.EndMode3D()
    rl.DrawRectangle(10,10,320,93,rl.Fade(rl.SKYBLUE,0.5))
    rl.DrawRectangleLines(10,10,320,93,rl.BLUE)
    rl.DrawText("Free camera default controls:", 20, 20, 10, rl.BLACK)
    rl.DrawText("- Mouse Wheel to Zoom in-out", 40, 40, 10, rl.DARKGRAY)
    rl.DrawText("- Mouse Wheel Pressed to Pan", 40, 60, 10, rl.DARKGRAY)
    rl.DrawText("- Z to zoom to (0, 0, 0)", 40, 80, 10, rl.DARKGRAY);
    rl.EndDrawing()
end
rl.CloseWindow()
