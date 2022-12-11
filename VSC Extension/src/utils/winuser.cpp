#include <Windows.h>
#include <string>
#include <node.h>

HWND vscWindow;
HWND studioWindow;

bool isVSC(char* title, std::string window)
{
    return std::string(title).find(window + " - Visual Studio Code") != std::string::npos;
}

bool isStudio(char* title)
{
    return std::string(title).find("Roblox Studio") != std::string::npos;
}

HWND getWindow(std::string window)
{
    for (HWND hwnd = GetTopWindow(NULL); hwnd != NULL; hwnd = GetNextWindow(hwnd, GW_HWNDNEXT))
    {
        if (!IsWindowVisible(hwnd))
            continue;

        int length = GetWindowTextLength(hwnd);

        if (length == 0)
            continue;

        char* title = new char[length + 1];

        GetWindowText(hwnd, title, length + 1);

        if (std::string(title) == "Program Manager")
        {
            delete [] title;
            continue;
        }

        if (!window.empty())
        {
            if (isVSC(title, window))
            {
                delete [] title;
                return hwnd;
            }
        }
        else
        {
            if (isStudio(title)) 
            {
                delete [] title;
                return hwnd;
            }
        }

        delete [] title;
    }

    return NULL;
}

namespace winuser {
    using v8::FunctionCallbackInfo;
    using v8::Integer;
    using v8::String;
    using v8::Object;
    using v8::Value;
    using v8::Local;

    void showVSC(const FunctionCallbackInfo<Value>& args)
    {
        if (!vscWindow)
        {
            String::Utf8Value v8String(args.GetIsolate(), args[0]);
            std::string stdString(*v8String);
            vscWindow = getWindow(stdString);
        }

        bool pressed = false;

        if ((GetAsyncKeyState(0x12) & 0x8000) == 0)
        {
            keybd_event(0x12, 0, 0x0001 | 0, 0);
            pressed = true;
        }

        ShowWindow(vscWindow, 3);
        SetForegroundWindow(vscWindow);

        if (pressed)
            keybd_event(0x12, 0, 0x0001 | 0x0002, 0);
    }

    void showStudio(const FunctionCallbackInfo<Value>& args)
    {
        if (!studioWindow)
            studioWindow = getWindow("");

        bool pressed = false;

        if ((GetAsyncKeyState(0x12) & 0x8000) == 0)
        {
            keybd_event(0x12, 0, 0x0001 | 0, 0);
            pressed = true;
        }

        ShowWindow(studioWindow, 3);
        SetForegroundWindow(studioWindow);

        if (pressed)
            keybd_event(0x12, 0, 0x0001 | 0x0002, 0);

        keybd_event(args[0].As<Integer>()->Value(), 0, 0x0001 | 0, 0);
        keybd_event(args[0].As<Integer>()->Value(), 0, 0x0001 | 0x0002, 0);
    }

    void Initialize(Local<Object> exports)
    {
        NODE_SET_METHOD(exports, "showVSC", showVSC);
        NODE_SET_METHOD(exports, "showStudio", showStudio);
    }

    NODE_MODULE(NODE_GYP_MODULE_NAME, Initialize)
}

//node-gyp configure
//node-gyp build --target=19.1.8 --arch=x64 --dist-url=https://electronjs.org/headers
//node-gyp rebuild --target=19.1.8 --arch=x64 --dist-url=https://electronjs.org/headers