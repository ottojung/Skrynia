#![allow(dead_code)]

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn open_disk(path_ptr: u32, path_len: u32, mode: u32) -> i32;
    pub fn truncate(fd: i32, size: u64) -> i32;
    pub fn seek(fd: i32, offset: u64) -> i32;
    pub fn read(fd: i32, dst_ptr: u32, len: u32) -> i32;
    pub fn write(fd: i32, src_ptr: u32, len: u32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn mkdir(path_ptr: u32, path_len: u32, mode: u32) -> i32;
    pub fn exists(path_ptr: u32, path_len: u32) -> i32;
    pub fn delete(path_ptr: u32, path_len: u32) -> i32;
    pub fn rename(old_ptr: u32, old_len: u32, new_ptr: u32, new_len: u32) -> i32;
    pub fn open_directory(path_ptr: u32, path_len: u32) -> i32;
    pub fn next_entry(dir: i32, dst_ptr: u32, cap: u32) -> i32;
    pub fn is_dir(path_ptr: u32, path_len: u32) -> i32;
    pub fn open_ram() -> i32;
    pub fn cas_ram_u64(ram_fd: i32, off: u64, exp: u64, des: u64, observed_ptr: u32) -> i32;
    pub fn sync(fd: i32) -> i32;
    pub fn now() -> i64;
    pub fn secure_random_bits(dst_ptr: u32, bits: u32) -> i32;
    pub fn eval(
        code_ptr: u32,
        code_len: u32,
        input_ptr: u32,
        input_len: u32,
        cb_table_idx: i32,
        out_ptr: u32,
        out_cap: u32,
    ) -> i32;
    pub fn eval_output_len() -> i32;
}
