package.path = "./raylib/?.lua;" .. package.path
local rl = require 'raylib'
local ffi = require 'ffi'


local screenWidth = 800
local screenHeight = 450

rl.InitWindow(screenWidth, screenHeight, "a 3d example")

local position = rl.Vector3(0.0, 2.0, 4.0)
local target = rl.Vector3(0.0, 2.0, 0.0)
local up = rl.Vector3(0.0, 1.0, 0.0)
local camera = rl.new("Camera3D[1]", rl.Camera3D(position, target, up, 60.0, rl.CAMERA_PERSPECTIVE))
local camera_mode = rl.CAMERA_FIRST_PERSON
local heights = {}
local positions = {}
local colors = {}
for i = 0, 20 do
    heights[i] = rl.GetRandomValue(1, 12)
    positions[i] = rl.Vector3(rl.GetRandomValue(-15, 15), heights[i] / 2.0, rl.GetRandomValue(-15, 15))
    colors[i] = rl.Color(rl.GetRandomValue(20, 255), rl.GetRandomValue(10, 55), 30, 255)
end
rl.DisableCursor()
rl.SetTargetFPS(60)
while not rl.WindowShouldClose() do
    if (rl.IsKeyPressed(rl.KEY_ONE)) then
        camera_mode = rl.CAMERA_FREE
        camera[0].up = rl.Vector3(0, 1, 0)
    elseif (rl.IsKeyPressed(rl.KEY_TWO)) then
        camera_mode = rl.CAMERA_FIRST_PERSON;
        camera[0].up = rl.Vector3(0, 1, 0)
    elseif (rl.IsKeyPressed(rl.KEY_THREE)) then
        camera_mode = rl.CAMERA_THIRD_PERSON
        camera[0].up = rl.Vector3(0.0, 1.0, 0.0)
    elseif (rl.IsKeyPressed(rl.KEY_FOUR)) then
        camera_mode = rl.CAMERA_ORBITAL
        camera[0].up = rl.Vector3(0.0, 1.0, 0.0)
    end
    if (rl.IsKeyPressed(rl.KEY_P)) then
        if (camera[0].projection == rl.CAMERA_PERSPECTIVE) then
            camera_mode = rl.CAMERA_THIRD_PERSON
            camera[0].position = rl.Vector3(0.0, 2.0, -100.0);
            camera[0].target = rl.Vector3(0.0, 2.0, 0.0);
            camera[0].up = rl.Vector3(0.0, 1.0, 0.0);
            camera[0].projection = rl.CAMERA_ORTHOGRAPHIC;
            camera[0].fovy = 20.0;
            rl.CameraYaw(camera, -135 * rl.DEG2RAD, true)
            rl.CameraPitch(camera, -45 * rl.DEG2RAD, true, true, false);
        elseif (camera[0].projection == rl.CAMERA_ORTHOGRAPHIC) then
            camera_mode = rl.CAMERA_THIRD_PERSON
            camera[0].position = rl.Vector3(0, 2.0, 10.0);
            camera[0].target = rl.Vector3(0, 2, 0.0);
            camera[0].up = rl.Vector3(0, 1, 0.0);
            camera[0].projection = rl.CAMERA_PERSPECTIVE;
            camera[0].fovy = 60.0;
        end
    end
    rl.UpdateCamera(camera, camera_mode)
    rl.BeginDrawing()
    rl.ClearBackground(rl.RAYWHITE)
    rl.BeginMode3D(camera[0])
    rl.DrawPlane(rl.Vector3(0.0, 0.0, 0.0), rl.Vector2(32.0, 32.0), rl.LIGHTGRAY)
    rl.DrawCube(rl.Vector3(-16, 2.5, 0), 1, 5, 32, rl.BLUE)
    rl.DrawCube(rl.Vector3(16, 2.5, 0), 1, 5, 32, rl.LIME)
    rl.DrawCube(rl.Vector3(0, 2.5, 16), 32, 5, 1, rl.GOLD)
    for i = 0, 20 do
        rl.DrawCube(positions[i], 2.0, heights[i], 2.0, colors[i])
        rl.DrawCubeWires(positions[i], 2.0, heights[i], 2.0, rl.MAROON)
    end
    if (camera_mode == rl.CAMERA_THIRD_PERSON) then
        rl.DrawCube(camera[0].target, 0.5, 0.5, 0.5, rl.PURPLE)
        rl.DrawCubeWires(camera[0].target, 0.5, 0.5, 0.5, rl.DARKPURPLE)
    end
    rl.EndMode3D()
    rl.DrawRectangle(5, 5, 330, 100, rl.Fade(rl.SKYBLUE, 0.5))
    rl.DrawRectangleLines(5, 5, 330, 100, rl.BLUE)
    rl.DrawText("Camera controls:", 15, 15, 10, rl.BLACK)
    rl.DrawText("- Move keys: W, A, S, D, Space, Left-Ctrl", 15, 30, 10, rl.BLACK)
    rl.DrawText("- Look around: arrow keys or mouse", 15, 45, 10, rl.BLACK);
    rl.DrawText("- Camera mode keys: 1, 2, 3, 4", 15, 60, 10, rl.BLACK)
    rl.DrawText("- Zoom keys: num-plus, num-minus or mouse scroll", 15, 75, 10, rl.BLACK)
    rl.DrawText("- Camera projection key: P", 15, 90, 10, rl.BLACK)

    rl.DrawRectangle(600, 5, 195, 100, rl.Fade(rl.SKYBLUE, 0.5));
    rl.DrawRectangleLines(600, 5, 195, 100, rl.BLUE);
    rl.DrawText("Camera status:", 610, 15, 10, rl.BLACK);
    rl.DrawText(
        rl.TextFormat("- Position: (%06.3f, %06.3f, %06.3f)", camera[0].position.x, camera[0].position.y,
            camera[0].position.z),
        610, 60, 10, rl.BLACK)
    rl.DrawText(
        rl.TextFormat("- Target: (%06.3f, %06.3f, %06.3f)", camera[0].target.x, camera[0].target.y, camera[0].target.z),
        610, 75, 10, rl.BLACK);
    rl.DrawText(rl.TextFormat("- Up: (%06.3f, %06.3f, %06.3f)", camera[0].up.x, camera[0].up.y, camera[0].up.z), 610, 90,
        10,
        rl.BLACK);

    rl.EndDrawing()
end
rl.CloseWindow()
