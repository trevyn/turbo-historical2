fn main() {
 let triple = std::env::var("TARGET").unwrap();
 if triple != "wasm32-unknown-unknown" {
  println!("cargo:rustc-link-search=rclone/build");
  println!("cargo:rustc-link-arg-bins=-lrclone-{}", triple);
  if triple.ends_with("darwin") {
   println!("cargo:rustc-link-arg-bins=-framework");
   println!("cargo:rustc-link-arg-bins=CoreFoundation");
   println!("cargo:rustc-link-arg-bins=-framework");
   println!("cargo:rustc-link-arg-bins=Security");
  }
  println!("cargo:rerun-if-changed=rclone/build/librclone-{}.a", triple);
 }
}
