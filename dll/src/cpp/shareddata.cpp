#include <windows.h>
#include <iostream>
using namespace std;

#pragma data_seg("SHARED")
	extern "C" HHOOK gHook = NULL;                // マウスフック識別用のハンドル
	extern "C" LONG gLastX = 0;
	extern "C" LONG gLastY = 0;
	extern "C" unsigned char gDirectionChain[32] = {};
#pragma data_seg()
