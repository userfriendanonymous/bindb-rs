
fn main() {

}

struct R<'a>(&'a mut String);

fn idk<'a, 'b> (data: &'b R<'a>) -> &'a mut String
where 'a: 'b {
    data.0
}