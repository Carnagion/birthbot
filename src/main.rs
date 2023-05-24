fn main() {
    use birthbot::prelude::*;
    use chrono::prelude::*;
    use mongodm::prelude::*;
    let birthday = Birthday(DateTime::default());
    let bson = to_bson(&birthday).unwrap();
    println!("{:?}", bson.as_str());
}
