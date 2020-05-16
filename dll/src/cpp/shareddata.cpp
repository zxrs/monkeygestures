#include <windows.h>

#pragma data_seg("SHARED")
	extern "C" HHOOK gHook = NULL;
	extern "C" LONG gLastX = 0;
	extern "C" LONG gLastY = 0;
#pragma data_seg()
