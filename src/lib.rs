use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    task::Waker,
};
use task::{Scope, Task};

pub mod task;

struct Queue<M> {
    events: VecDeque<Rc<dyn Fn(&mut M) -> Option<()>>>,
    waker: Option<Waker>,
}

pub struct System<M, F, S> {
    pub model: M,
    make_task: F,
    state: Option<S>,
    cx: Scope<M>,
    queue: Rc<RefCell<Queue<M>>>,
}

impl<M: 'static, F, S> System<M, F, S> {
    pub fn new(model: M, make_task: F) -> Self {
        let queue = Rc::new(RefCell::new(Queue {
            events: VecDeque::new(),
            waker: None,
        }));

        Self {
            model,
            make_task,
            state: None,
            queue: queue.clone(),
            cx: Scope {
                waker: Rc::new(move |f| {
                    let mut q = queue.borrow_mut();
                    q.events.push_back(f);
                    if let Some(waker) = q.waker.take() {
                        waker.wake();
                    }
                }),
                contexts: RefCell::new(HashMap::new()),
            },
        }
    }

    pub fn build<T>(&mut self) -> T::Output
    where
        F: FnMut(&mut M) -> T,
        T: Task<M, State = S>,
    {
        let mut task = (self.make_task)(&mut self.model);
        let (output, state) = task.build(&self.cx, &mut self.model);
        self.state = Some(state);
        output
    }

    pub fn rebuild<T>(&mut self) -> T::Output
    where
        F: FnMut(&mut M) -> T,
        T: Task<M, State = S>,
    {
        let mut task = (self.make_task)(&mut self.model);
        task.rebuild(&self.cx, &mut self.model, self.state.as_mut().unwrap())
    }

    pub async fn update(&mut self) {
        futures::future::poll_fn(|cx| {
            let mut queue = self.queue.borrow_mut();
            let mut is_ready = false;
            while let Some(update) = queue.events.pop_front() {
                update(&mut self.model);
                is_ready = true;
            }
            if is_ready {
                std::task::Poll::Ready(())
            } else {
                queue.waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        })
        .await
    }
}
