use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let install_dir = out_dir.join("install");
    let build_dir = out_dir.join("build");

    if !install_dir.exists() {
        fs::create_dir_all(&build_dir).unwrap();
        fs::create_dir_all(&install_dir).unwrap();

        // 1. Build zlib (libmagic dependency)
        let zlib_version = "1.3.1";
        let zlib_tarball = format!("zlib-{}.tar.gz", zlib_version);
        let zlib_url = format!("https://github.com/madler/zlib/releases/download/v{}/{}", zlib_version, zlib_tarball);
        
        println!("cargo:warning=Downloading zlib from {}", zlib_url);
        Command::new("curl").arg("-L").arg("-O").arg(&zlib_url).current_dir(&build_dir).status().unwrap();
        Command::new("tar").arg("xzf").arg(&zlib_tarball).current_dir(&build_dir).status().unwrap();
        
        let zlib_src = build_dir.join(format!("zlib-{}", zlib_version));

        // OpenSSF Hardening Flags
        // https://best.openssf.org/Compiler-Hardening-Guides/Compiler-Options-Hardening-Guide-for-C-and-C++.html
        let cflags = "-O2 -D_FORTIFY_SOURCE=2 -fstack-protector-strong -fstack-clash-protection -fcf-protection -fPIC -Wall -Wformat -Wformat-security";
        let ldflags = "-Wl,-z,relro -Wl,-z,now -Wl,-z,noexecstack";

        unsafe {
            env::set_var("CFLAGS", cflags);
            env::set_var("LDFLAGS", ldflags);
        }

        Command::new("./configure").arg(format!("--prefix={}", install_dir.display())).arg("--static").current_dir(&zlib_src).status().unwrap();
        Command::new("make").arg("-j").arg(num_cpus::get().to_string()).current_dir(&zlib_src).status().unwrap();
        Command::new("make").arg("install").current_dir(&zlib_src).status().unwrap();

        // 2. Build libmagic
        let magic_version = "5.46";
        let magic_tarball = format!("file-{}.tar.gz", magic_version);
        let magic_url = format!("https://astron.com/pub/file/{}", magic_tarball);
        
        println!("cargo:warning=Downloading libmagic from {}", magic_url);
        Command::new("curl").arg("-L").arg("-O").arg(&magic_url).current_dir(&build_dir).status().unwrap();
        Command::new("tar").arg("xzf").arg(&magic_tarball).current_dir(&build_dir).status().unwrap();
        
        let magic_src = build_dir.join(format!("file-{}", magic_version));
        
        // Point libmagic to our local zlib AND apply hardening
        unsafe {
            env::set_var("CFLAGS", format!("{} -I{}/include", cflags, install_dir.display()));
            env::set_var("LDFLAGS", format!("{} -L{}/lib", ldflags, install_dir.display()));
        }
        
        Command::new("./configure")
            .arg(format!("--prefix={}", install_dir.display()))
            .arg("--disable-shared")
            .arg("--enable-static")
            .arg("--disable-dependency-tracking")
            .arg("--enable-zlib")
            .arg("--disable-bzlib")
            .arg("--disable-xzlib")
            .arg("--disable-lzlib")
            .arg("--disable-zstdlib")
            .current_dir(&magic_src)
            .status()
            .unwrap();
            
        Command::new("make").arg("-j").arg(num_cpus::get().to_string()).current_dir(&magic_src).status().unwrap();
        Command::new("make").arg("install").current_dir(&magic_src).status().unwrap();
    }

    println!("cargo:rustc-link-search=native={}/lib", install_dir.display());
    println!("cargo:rustc-link-lib=static=magic");
    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rerun-if-changed=build.rs");
}
