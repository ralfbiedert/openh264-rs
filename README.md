
# OpenH264 Rust API

Idiomatic(*) and low-level bindings for OpenH264.  

*decoder only for now, encoder PRs welcome.


# FAQ

- **How does `openh264-sys2` differ from `openh264-sys`?**

  We directly ship OpenH264 source code and provide simple, hand-crafted compilation via `cc` in `build.rs`. Our`openh264-sys2` crate should compile via `cargo build` out of the box on most platforms, and cross-compile via `cargo build --target ...` as 
  long as the environment variable `CC` is properly set. 

  
- **I need to fix an important OpenH264 security hole, how can I update the library?**

  Cisco's OpenH264 library is contained in `openh264-sys2/upstream`. Updating is as simple as [pulling their latest source](https://github.com/cisco/openh264), 
  copying it into that directory, and manually removing all "resource" files. We probably should have a script to strip that folder automatically ...  
  

- **I heard Rust is super-safe, will this make decoding my videos safe too?**

  No. Below a thin Rust layer we rely on a _very complex_ C library, and an equally complex standard. Apart from Rust being a 
  much nicer language to work with, depending on this  project will give you _no_ additional safety guarantees as far as video 
  handling is concerned. FYI, this is _not_ making a statement about OpenH264, but about the realities of securing +50k lines 
  of C against attacks.    


- **Feature X is missing or broken, will you fix it?**

  Right now I only have time implementing what I need. However, I will gladly accept PRs either extending the APIs, or fixing bugs; see below.



# Contributing

PRs are very welcome. Feel free to submit PRs and fixes right away. You can open Issues if you want to discuss things, but due to time restrictions on my side the project will have to rely on people contributing. 



# License

- OpenH264 core library is [BSD-2](openh264-sys2/upstream/LICENSE), Cisco.
- Wrapper code is [BSD-2](https://opensource.org/licenses/BSD-2-Clause), Ralf Biedert. 
