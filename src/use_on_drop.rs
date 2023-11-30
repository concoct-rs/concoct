use crate::LocalContext;

pub fn use_on_drop(on_drop: impl FnMut() + 'static) {
    let cx = LocalContext::current();
    let mut scope = cx.scope.borrow_mut();
    if scope.on_drops.borrow().get(scope.drops_idx).is_none() {
        scope.on_drops.borrow_mut().push(Box::new(on_drop));
        scope.drops_idx += 1;
    }
}
