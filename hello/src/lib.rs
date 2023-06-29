use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct User {
    name: String,
    age: u32,
    money: f32,
}

#[no_mangle]
fn main(input_ptr: u32) {
    // Read object from memory
    let input = unsafe {
        let len = *(input_ptr as *const u32);
        let bytes = (input_ptr + 4) as *const u8;
        let slice = core::slice::from_raw_parts(bytes, len as usize);
        rmp_serde::from_slice::<User>(slice)
    };
    
    println!("{input:?}");
}
