#![feature(new_uninit)]

#[derive(Clone, Debug)]
#[repr(C)] // Not necessary but order is maintained as declared
// This struct is self-referential so it needs to use Pin as
// Astruct::op_a_u32 should be Some(&Astruct::a_u32):
//    https://doc.rust-lang.org/std/pin/index.html
struct Astruct<'m> {
    a_u8: u8,
    a_u32: u32,
    op_a_u32: Option<&'m u32>, // Point to Astruct::a_u32
}

// op_a_u32 cannot be initialized safely to a_u32
fn test_box_astruct<'m>() -> Box<Astruct<'m>> {
    Box::<Astruct>::new(Astruct {
        a_u8: 1,
        a_u32: 123,
        op_a_u32: None,
    })
}

// Initialize op_a_u32 using unsafe pointers,
fn test_maybe_uninit_astruct<'m>() -> Box<Astruct<'m>> {
    let mut uas = Box::<Astruct>::new_uninit();

    unsafe {
        (*uas.as_mut_ptr()).a_u8 = 4;
        (*uas.as_mut_ptr()).a_u32 = 456;
        (*uas.as_mut_ptr()).op_a_u32 = Some(&(*uas.as_mut_ptr()).a_u32);
        return uas.assume_init();
    }
}

// Using new_zeroed "is/maybe" safe in as a None pointer is a pointer with a value of zero
// at least on some machines
fn test_maybe_uninit_zeroed_astruct<'m>() -> Box<Astruct<'m>> {
    let mut uas = Box::<Astruct>::new_zeroed();

    unsafe {
        (*uas.as_mut_ptr()).a_u8 = 4;
        //(*uas.as_mut_ptr()).a_u32 = 456;
        //(*uas.as_mut_ptr()).op_a_u32 = Some(&(*uas.as_mut_ptr()).a_u32);
        return uas.assume_init();
    }
}

// Simple example of using new_uninit from the documentation:
//   https://doc.rust-lang.org/std/boxed/struct.Box.html#method.new_uninit
fn test_new_uninit() -> u32 {
    let mut five = Box::<u32>::new_uninit();

    unsafe {
        // Initializeing
        five.as_mut_ptr().write(5);

        // So we can "safely" assume it was initialized properly
        return *five.assume_init();
    }
}

// Example that MaybeUninit is very very very dangerous
//   https://doc.rust-lang.org/std/mem/union.MaybeUninit.html
// This is definitely NOT initialized and CANNOT be used!
fn test_definitly_not_initialized_using_maybe_uninit() -> &'static mut Vec<u32> {
    use std::mem::MaybeUninit;

    let mut x = MaybeUninit::<Vec<u32>>::uninit();
    let x_vec = unsafe { &mut *x.as_mut_ptr() };

    x_vec
}

fn main() {
    println!("five={}", test_new_uninit());

    // Also totally uninitialized, printed:
    //   $ cargo run
    //   Compiling expr-builder-pattern v0.1.0 (/home/wink/prgs/rust/myrepos/expr-builder-pattern)
    //       Finished dev [unoptimized + debuginfo] target(s) in 0.14s
    //       Running `target/debug/expr-builder-pattern`
    //   five=5
    //   test_maybe_uninit_vec: len=94209313899024
    println!("Definitly uninitialized test_maybe_uninit_vec: len={}", test_definitly_not_initialized_using_maybe_uninit().len());

    // Manually initialize Astruct
    let astruct = Astruct {
        a_u8: 1,
        a_u32: 321,
        op_a_u32: None,
    };
    println!("astruct: {:p} {:?}", &astruct, astruct);

    let mut bas = test_box_astruct();
    bas.op_a_u32 = Some(&bas.a_u32);
    println!("test_box_astruct: &bas{{:p}}={:p} bas{{:p}}={:p} &*bas{{:p}}={:p} bas{{:?}}={:?}", &bas, bas, &*bas, bas);

    let x = test_maybe_uninit_astruct();
    println!(
        r#"test_maybe_uninit_astruct: &x{{:p}}={:p} &*x{{:p}}={:p} &x.a_u8{{:p}}={:p} &x.a_u32{{:p}}={:p} &x.op_a_u32{{:p}}={:p} (&*x).op_a_u32.unwrap{{:p}}={:p}"#,
        &x, &*x, &x.a_u8, &x.a_u32, &x.op_a_u32, (&*x).op_a_u32.unwrap()
    );
    assert_eq!(&x.a_u32, (&*x).op_a_u32.unwrap());

    // Here is the above using explicit raw pointers:
    let p_a_u8 = &x.a_u8 as *const u8;
    let p_a_u32 = &x.a_u32 as *const u32;
    let p_op_a_u32 = &x.op_a_u32 as *const Option<&'static u32>;
    println!(
        r#"Addresses of the fields of x on the heap: p_a_u8={:p} p_a_u32={:p} p_op_a_u32={:p}"#,
        p_a_u8, p_a_u32, p_op_a_u32
    );
    unsafe {
        let raw_ptr = p_op_a_u32 as *const usize;
        println!("Print the address in x.op_a_use aka p_op_a_u32 using *raw_ptr=0x{:x}", *raw_ptr);
    }

    let mut z = test_maybe_uninit_zeroed_astruct();
    println!(r#"This happens to work because test_maybe_uninit_zeroed_astruct: z={:#?}"#, z);
    z.op_a_u32 = Some(&z.a_u32);
    println!(r#"This happens to work because test_maybe_uninit_zeroed_astruct: after initing z.op_a_u32 z={:#?}"#, z);

    // Self referental structure without using Option, as with Astruct::op_a_u32
    // Xstruct::p points to Xstruct::f os needs to use Pin:
    //    https://doc.rust-lang.org/std/pin/index.html
    #[derive(Debug)]
    #[repr(C)] // Not necessary but order is maintained as declared
    struct Xstruct<'x> {
        f: u32,
        p: &'x u32,
    }

    let mut ux = Box::<Xstruct>::new_uninit();
    unsafe {
        (*ux.as_mut_ptr()).f = 47;
        (*ux.as_mut_ptr()).p = &(*ux.as_mut_ptr()).f;
    }
    let ux = unsafe { ux.assume_init() };

    println!("ux={:?}", ux);
    println!("&ux={:p} &*ux={:p}", &ux, &*ux);
    println!("&ux.f={:p}", &ux.f);
    println!("&ux.p={:p}", &ux.p);
}
