# Operations

## `https://os.phil-opp.com/`

```shell
cargo new blog_os --bin --edition 2018
rustup target add thumbv7em-none-eabihf
cargo build --target thumbv7em-none-eabihf

# Linux
cargo rustc -- -C link-arg=-nostartfiles
# Windows
cargo rustc -- -C link-args="/ENTRY:_start /SUBSYSTEM:console"
# macOS
cargo rustc -- -C link-args="-e __start -static -nostartfiles"

cargo build --target x86_64-blog_os.json

cargo install bootimage
cargo bootimage
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin

qemu-img dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX
# for VBOX
qemu-img convert -f raw -O vdi target/x86_64-blog_os/debug/bootimage-blog_os.bin target/x86_64-blog_os/debug/bootimage-blog_os.vdi

cargo test 
cargo test --test should_panic
```