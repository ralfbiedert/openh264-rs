bindgen ^
    ..\upstream\codec\api\svc\codec_api.h ^
    --generate-block ^
    --no-layout-tests ^
    --no-prepend-enum-name ^
    --default-enum-style rust ^
    --with-derive-eq ^
    --with-derive-default ^
    --use-array-pointers-in-arguments ^
    -o ../src/generated.rs

pause