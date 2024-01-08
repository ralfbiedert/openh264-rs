REM Using Bindgen 0.58.1

SET HEADER=..\openh264-sys2\upstream\codec\api\wels\codec_api.h

bindgen ^
    %HEADER% ^
    --generate-block ^
    --no-layout-tests ^
    --no-prepend-enum-name ^
    --with-derive-eq --with-derive-default --with-derive-hash --with-derive-ord ^
    --use-array-pointers-in-arguments ^
    --generate types ^
    -o ../openh264-sys2/src/generated/types.rs

bindgen ^
    %HEADER% ^
    --generate-block ^
    --no-layout-tests ^
    --no-prepend-enum-name ^
    --with-derive-eq --with-derive-default --with-derive-hash --with-derive-ord ^
    --use-array-pointers-in-arguments ^
    --generate vars ^
    -o ../openh264-sys2/src/generated/consts.rs

bindgen ^
    %HEADER% ^
    --generate-block ^
    --no-layout-tests ^
    --no-prepend-enum-name ^
    --with-derive-eq --with-derive-default --with-derive-hash --with-derive-ord ^
    --use-array-pointers-in-arguments ^
    --raw-line "use super::types::*;" ^
    --generate functions ^
    -o ../openh264-sys2/src/generated/fns_source.rs

bindgen ^
    %HEADER% ^
    --generate-block ^
    --no-layout-tests ^
    --no-prepend-enum-name ^
    --with-derive-eq --with-derive-default --with-derive-hash --with-derive-ord ^
    --use-array-pointers-in-arguments ^
    --raw-line "use super::types::*;" ^
    --dynamic-loading APILoader ^
    --generate functions ^
    -o ../openh264-sys2/src/generated/fns_libloading.rs

pause