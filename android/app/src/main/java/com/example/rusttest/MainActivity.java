package com.example.rusttest;

import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("rust_test_app");
    }
}
