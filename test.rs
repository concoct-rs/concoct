fn a() {
    b();
    
    c();
}

fn b() {}

#[inline]
fn c() {}
