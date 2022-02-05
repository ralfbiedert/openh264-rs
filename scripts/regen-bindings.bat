REM Using Bindgen 0.58.1

bindgen ^
    ..\openh264-sys2\upstream\codec\api\svc\codec_api.h ^
    --generate-block ^
    --no-layout-tests ^
    --no-prepend-enum-name ^
    --with-derive-eq ^
    --with-derive-default ^
    --with-derive-hash ^
    --with-derive-ord ^
    --use-array-pointers-in-arguments ^
    -o ../openh264-sys2/src/generated.rs

pause