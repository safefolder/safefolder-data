
pub fn check_achiever_function(function_text: String) -> bool {
    let mut check = false;
    for function_item in FORMULA_FUNCTIONS {
        // let function_name_pieces = function_text.split("(");
        // let function_name_pieces: Vec<&str> = function_name_pieces.collect();
        // let function_name = function_name_pieces.0;
        let function_name = get_function_name(function_text);
        eprintln!("check_achiever_function :: formula function: {}", &function_item);
        eprintln!("check_achiever_function :: function_name: {}", &function_name);
        if function_item.to_lowercase() == function_name.to_string().to_lowercase() {
            check = true;
            eprintln!("check_achiever_function :: check: {}", &check);
            break
        }
    }
    return check;
}

pub fn get_function_name(function_text: String) -> String {
    let function_name_pieces = function_text.split("(");
    let function_name_pieces: Vec<&str> = function_name_pieces.collect();
    let function_name = function_name_pieces.0;
    return function_name
}