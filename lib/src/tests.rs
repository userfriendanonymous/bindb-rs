use crate::storage;

macro_rules! open_file {
    ($name: expr) => {
        std::fs::File::options().read(true).write(true).create(true).open($name).unwrap()
    };
}

fn init() {
    std::fs::remove_dir_all("./local").unwrap();
    std::fs::create_dir("./local").unwrap();
    std::env::set_var("RUST_BACKTRACE", "1");
}

binbuf::fixed! {
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TestEntry1 {
        #[lens(buf_idx)]
        idx: u64,
        #[lens(buf_opt)]
        opt: Option<bool>,
    }

    buf! { pub struct TestEntry1Buf<P>(TestEntry1, P); }

    impl I for TestEntry1 {
        type Buf<P> = TestEntry1Buf<P>;
    }
    impl Code for TestEntry1 {}
}

// #[test]
pub fn fixed_test1() {
    init();
    let mut db = unsafe {
        storage::Fixed::create(
            open_file!("./local/fixed1"),
            10
        )
    }.unwrap();

    let entry = TestEntry1 {
        idx: 5,
        opt: Some(false),
    };
    let id = db.add(&entry).unwrap();
    let output_entry = db.get(id);
    println!("{:?}", &output_entry);
    assert_eq!(entry, output_entry);

    db.remove_last().unwrap();
    assert_eq!(db.len(), 0);

    let entry = TestEntry1 {
        idx: 2077,
        opt: None,
    };
    let id = db.add(&entry).unwrap();
    let output_entry = db.get(id);
    println!("{:?}", &output_entry);
    assert_eq!(entry, output_entry);
    assert_eq!(db.len(), 1);
}

// #[test]
pub fn dynamic_test1() {
    init();
    let mut db = unsafe {
        storage::Dynamic::<String>::create(
            storage::Fixed::create(
                open_file!("./local/dynamic1_free_locations"),
                10,
            ).unwrap(),
            open_file!("./local/dynamic1"),
            10,
        )
    }.unwrap();

    let entry = "Hello from bindb!".to_string();
    let id = db.add(&entry).unwrap();
    let id1 = id;
    let out = unsafe { db.get(id) };
    println!("{}", &out);
    assert_eq!(&entry, &out);

    let entry = "What is up everyone? I'm feeling good today. How are you?".to_string();
    let id = db.add(&entry).unwrap();
    let out = unsafe { db.get(id) };
    println!("{}", &out);
    assert_eq!(&entry, &out);

    unsafe { db.remove(id1) }.unwrap();

    let entry = "Hello again!".to_string();
    let id = db.add(&entry).unwrap();
    let out = unsafe { db.get(id1) };
    println!("{}", &out);
    assert_eq!(&entry, &out);
}

// #[test]
pub fn indexed_dynamic_test1() {
    init();
    let mut db = unsafe {
        storage::IndexedDynamic::<String>::open(
            storage::Dynamic::create(
                storage::Fixed::create(
                    open_file!("./local/indexed_dynamic1_free_locations"),
                    10,
                ).unwrap(),
                open_file!("./local/indexed_dynamic1"),
                10,
            ).unwrap(),
            storage::Fixed::create(
                open_file!("./local/indexed_dynamic1_indices"),
                10
            ).unwrap(),
            storage::Fixed::create(
                open_file!("./local/indexed_dynamic1_free_ids"),
                10
            ).unwrap(),
        ).unwrap()
    };

    let entry = "What is up everyone? I'm feeling good today. How are you?".to_string();
    let id = db.add(&entry).unwrap();
    let out = unsafe { db.get(id) };
    println!("{}", &out);
    assert_eq!(&entry, &out);

    unsafe { db.remove(id).unwrap() };
}

// #[test]
pub fn single_test1() {
    init();
    let entry = "What's up!?".to_string();
    let mut db = unsafe {
        storage::Single::create(open_file!("./local/single1"), &entry).unwrap()
    };

    assert_eq!(&db.get(), &entry);

    let entry = "New value...".to_string();
    db.set(&entry).unwrap();
    assert_eq!(&db.get(), &entry);
}

// #[test]
pub fn binary_tree_test1() {
    use binbuf::impls::{ArbNum, arb_num};
    init();
    let entry = "What's up!?".to_string();
    let mut db = unsafe {
        storage::BinaryTree::<ArbNum<4, u64>, i32, TestEntry1>::create(
            storage::Fixed::create(open_file!("./local/binary_tree1_nodes"), 10).unwrap(),
            storage::Fixed::create(open_file!("./local/binary_tree_free_ids"), 10).unwrap(),
            open_file!("./local/binary_tree1_header")
        ).unwrap()
    };
    let entry = TestEntry1 {
        idx: 999,
        opt: Some(true),
    };
    db.add(&584, &entry).unwrap();

    assert_eq!(db.get(&555), None);
    assert_eq!(db.get(&584), Some(entry.clone()));

    db.add(&103, &entry).unwrap();
    db.remove(&584);

    assert_eq!(db.get(&584), None);
}