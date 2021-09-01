REM Using Bindgen 0.58.1

bindgen ^
    ..\upstream\codec\api\svc\codec_api.h ^
    --generate-block ^
    --no-layout-tests ^
    --no-prepend-enum-name ^
    --default-enum-style rust ^
    --with-derive-eq ^
    --with-derive-default ^
    --with-derive-hash ^
    --with-derive-ord ^
    --use-array-pointers-in-arguments ^
    -o ../src/generated.rs

pause