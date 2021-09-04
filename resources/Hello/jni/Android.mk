LOCAL_PATH := $(call my-dir)
# LOCAL_XOM := false


# ### build the main ###
include $(CLEAR_VARS)
LOCAL_MODULE := hello
LOCAL_SRC_FILES := hello.cpp

include $(BUILD_EXECUTABLE)