pub fn string_to_object_id(string_id: String) -> Result<[u8; 12], Box<dyn std::error::Error>> {
    let bytes_id = hex::decode(string_id)?;
    let mut output: [u8; 12] = [0; 12]; // Initialize a fixed-size array with zeros

    if bytes_id.len() == 12 {
        // If the Vec has at least 12 elements, copy the first 12 elements into the array
        output.copy_from_slice(&bytes_id[0..12]);
    } else {
        return Err("Invalid ObjectId length".into());
    }

    Ok(output)
}
