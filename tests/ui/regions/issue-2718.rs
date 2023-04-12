// run-pass
#![allow(dead_code)]
#![allow(unused_unsafe)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]

pub type Task = isize;

// tjc: I don't know why
pub mod pipes {
    use self::state::{empty, full, blocked, terminated};
    use super::Task;
    use std::mem::{forget, transmute};
    use std::mem::{replace, swap};
    use std::mem;
    use std::thread;
    use std::marker::Send;

    pub struct Stuff<T> {
        state: state,
        blocked_task: Option<Task>,
        payload: Option<T>
    }

    #[derive(PartialEq, Debug)]
    #[repr(isize)]
    pub enum state {
        empty,
        full,
        blocked,
        terminated
    }

    pub struct packet<T> {
        state: state,
        blocked_task: Option<Task>,
        payload: Option<T>
    }

    unsafe impl<T:Send> Send for packet<T> {}

    pub fn packet<T:Send>() -> *const packet<T> {
        unsafe {
            let p: *const packet<T> = mem::transmute(Box::new(Stuff{
                state: empty,
                blocked_task: None::<Task>,
                payload: None::<T>
            }));
            p
        }
    }

    mod crablangi {
      pub fn atomic_xchg(_dst: &mut isize, _src: isize) -> isize { panic!(); }
      pub fn atomic_xchg_acq(_dst: &mut isize, _src: isize) -> isize { panic!(); }
      pub fn atomic_xchg_rel(_dst: &mut isize, _src: isize) -> isize { panic!(); }
    }

    // We should consider moving this to ::std::unsafe, although I
    // suspect graydon would want us to use void pointers instead.
    pub unsafe fn uniquify<T>(x: *const T) -> Box<T> {
        mem::transmute(x)
    }

    pub fn swap_state_acq(dst: &mut state, src: state) -> state {
        unsafe {
            transmute(crablangi::atomic_xchg_acq(transmute(dst), src as isize))
        }
    }

    pub fn swap_state_rel(dst: &mut state, src: state) -> state {
        unsafe {
            transmute(crablangi::atomic_xchg_rel(transmute(dst), src as isize))
        }
    }

    pub fn send<T:Send>(mut p: send_packet<T>, payload: T) {
        let p = p.unwrap();
        let mut p = unsafe { uniquify(p) };
        assert!((*p).payload.is_none());
        (*p).payload = Some(payload);
        let old_state = swap_state_rel(&mut (*p).state, full);
        match old_state {
          empty => {
            // Yay, fastpath.

            // The receiver will eventually clean this up.
            unsafe { forget(p); }
          }
          full => { panic!("duplicate send") }
          blocked => {

            // The receiver will eventually clean this up.
            unsafe { forget(p); }
          }
          terminated => {
            // The receiver will never receive this. Rely on drop_glue
            // to clean everything up.
          }
        }
    }

    pub fn recv<T:Send>(mut p: recv_packet<T>) -> Option<T> {
        let p = p.unwrap();
        let mut p = unsafe { uniquify(p) };
        loop {
            let old_state = swap_state_acq(&mut (*p).state,
                                           blocked);
            match old_state {
              empty | blocked => { thread::yield_now(); }
              full => {
                let payload = replace(&mut p.payload, None);
                return Some(payload.unwrap())
              }
              terminated => {
                assert_eq!(old_state, terminated);
                return None;
              }
            }
        }
    }

    pub fn sender_terminate<T:Send>(p: *const packet<T>) {
        let mut p = unsafe { uniquify(p) };
        match swap_state_rel(&mut (*p).state, terminated) {
          empty | blocked => {
            // The receiver will eventually clean up.
            unsafe { forget(p) }
          }
          full => {
            // This is impossible
            panic!("you dun goofed")
          }
          terminated => {
            // I have to clean up, use drop_glue
          }
        }
    }

    pub fn receiver_terminate<T:Send>(p: *const packet<T>) {
        let mut p = unsafe { uniquify(p) };
        match swap_state_rel(&mut (*p).state, terminated) {
          empty => {
            // the sender will clean up
            unsafe { forget(p) }
          }
          blocked => {
            // this shouldn't happen.
            panic!("terminating a blocked packet")
          }
          terminated | full => {
            // I have to clean up, use drop_glue
          }
        }
    }

    pub struct send_packet<T:Send> {
        p: Option<*const packet<T>>,
    }

    impl<T:Send> Drop for send_packet<T> {
        fn drop(&mut self) {
            unsafe {
                if self.p != None {
                    let self_p: &mut Option<*const packet<T>> =
                        mem::transmute(&mut self.p);
                    let p = replace(self_p, None);
                    sender_terminate(p.unwrap())
                }
            }
        }
    }

    impl<T:Send> send_packet<T> {
        pub fn unwrap(&mut self) -> *const packet<T> {
            replace(&mut self.p, None).unwrap()
        }
    }

    pub fn send_packet<T:Send>(p: *const packet<T>) -> send_packet<T> {
        send_packet {
            p: Some(p)
        }
    }

    pub struct recv_packet<T:Send> {
        p: Option<*const packet<T>>,
    }

    impl<T:Send> Drop for recv_packet<T> {
        fn drop(&mut self) {
            unsafe {
                if self.p != None {
                    let self_p: &mut Option<*const packet<T>> =
                        mem::transmute(&mut self.p);
                    let p = replace(self_p, None);
                    receiver_terminate(p.unwrap())
                }
            }
        }
    }

    impl<T:Send> recv_packet<T> {
        pub fn unwrap(&mut self) -> *const packet<T> {
            replace(&mut self.p, None).unwrap()
        }
    }

    pub fn recv_packet<T:Send>(p: *const packet<T>) -> recv_packet<T> {
        recv_packet {
            p: Some(p)
        }
    }

    pub fn entangle<T:Send>() -> (send_packet<T>, recv_packet<T>) {
        let p = packet();
        (send_packet(p), recv_packet(p))
    }
}

pub mod pingpong {
    use std::mem;

    pub struct ping(::pipes::send_packet<pong>);

    unsafe impl Send for ping {}

    pub struct pong(::pipes::send_packet<ping>);

    unsafe impl Send for pong {}

    pub fn liberate_ping(p: ping) -> ::pipes::send_packet<pong> {
        unsafe {
            let _addr : *const ::pipes::send_packet<pong> = match &p {
              &ping(ref x) => { mem::transmute(x) }
            };
            panic!()
        }
    }

    pub fn liberate_pong(p: pong) -> ::pipes::send_packet<ping> {
        unsafe {
            let _addr : *const ::pipes::send_packet<ping> = match &p {
              &pong(ref x) => { mem::transmute(x) }
            };
            panic!()
        }
    }

    pub fn init() -> (client::ping, server::ping) {
        ::pipes::entangle()
    }

    pub mod client {
        use pingpong;

        pub type ping = ::pipes::send_packet<pingpong::ping>;
        pub type pong = ::pipes::recv_packet<pingpong::pong>;

        pub fn do_ping(c: ping) -> pong {
            let (sp, rp) = ::pipes::entangle();

            ::pipes::send(c, pingpong::ping(sp));
            rp
        }

        pub fn do_pong(c: pong) -> (ping, ()) {
            let packet = ::pipes::recv(c);
            if packet.is_none() {
                panic!("sender closed the connection")
            }
            (pingpong::liberate_pong(packet.unwrap()), ())
        }
    }

    pub mod server {
        use pingpong;

        pub type ping = ::pipes::recv_packet<pingpong::ping>;
        pub type pong = ::pipes::send_packet<pingpong::pong>;

        pub fn do_ping(c: ping) -> (pong, ()) {
            let packet = ::pipes::recv(c);
            if packet.is_none() {
                panic!("sender closed the connection")
            }
            (pingpong::liberate_ping(packet.unwrap()), ())
        }

        pub fn do_pong(c: pong) -> ping {
            let (sp, rp) = ::pipes::entangle();
            ::pipes::send(c, pingpong::pong(sp));
            rp
        }
    }
}

fn client(chan: pingpong::client::ping) {
    let chan = pingpong::client::do_ping(chan);
    println!("Sent ping");
    let (_chan, _data) = pingpong::client::do_pong(chan);
    println!("Received pong");
}

fn server(chan: pingpong::server::ping) {
    let (chan, _data) = pingpong::server::do_ping(chan);
    println!("Received ping");
    let _chan = pingpong::server::do_pong(chan);
    println!("Sent pong");
}

pub fn main() {
  /*
//    Commented out because of option::get error

    let (client_, server_) = pingpong::init();

    task::spawn {|client_|
        let client__ = client_.take();
        client(client__);
    };
    task::spawn {|server_|
        let server__ = server_.take();
        server(server_ˊ);
    };
  */
}
