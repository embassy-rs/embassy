//! Unsafe linked list.
//! Translated from ST's C by `c2rust` tool.

#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

use cortex_m::interrupt;

#[derive(Copy, Clone)]
#[repr(C, packed(4))]
pub struct LinkedListNode {
    pub next: *mut LinkedListNode,
    pub prev: *mut LinkedListNode,
}

impl Default for LinkedListNode {
    fn default() -> Self {
        LinkedListNode {
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        }
    }
}

impl LinkedListNode {
    pub unsafe fn init_head(mut listHead: *mut LinkedListNode) {
        (*listHead).next = listHead;
        (*listHead).prev = listHead;
    }

    pub unsafe fn is_empty(mut listHead: *mut LinkedListNode) -> bool {
        interrupt::free(|_| ((*listHead).next) == listHead)
    }

    pub unsafe fn insert_head(mut listHead: *mut LinkedListNode, mut node: *mut LinkedListNode) {
        interrupt::free(|_| {
            (*node).next = (*listHead).next;
            (*node).prev = listHead;
            (*listHead).next = node;
            (*(*node).next).prev = node;
        });
    }

    pub unsafe fn insert_tail(mut listHead: *mut LinkedListNode, mut node: *mut LinkedListNode) {
        interrupt::free(|_| {
            (*node).next = listHead;
            (*node).prev = (*listHead).prev;
            (*listHead).prev = node;
            (*(*node).prev).next = node;
        });
    }

    pub unsafe fn remove_node(mut node: *mut LinkedListNode) {
        interrupt::free(|_| {
            (*(*node).prev).next = (*node).next;
            (*(*node).next).prev = (*node).prev;
        });
    }

    pub unsafe fn remove_head(mut listHead: *mut LinkedListNode, mut node: *mut *mut LinkedListNode) {
        interrupt::free(|_| {
            *node = (*listHead).next;
            Self::remove_node((*listHead).next);
        });
    }

    pub unsafe fn remove_tail(mut listHead: *mut LinkedListNode, mut node: *mut *mut LinkedListNode) {
        interrupt::free(|_| {
            *node = (*listHead).prev;
            Self::remove_node((*listHead).prev);
        });
    }

    pub unsafe fn insert_node_after(mut node: *mut LinkedListNode, mut ref_node: *mut LinkedListNode) {
        interrupt::free(|_| {
            (*node).next = (*ref_node).next;
            (*node).prev = ref_node;
            (*ref_node).next = node;
            (*(*node).next).prev = node;
        });
    }

    pub unsafe fn insert_node_before(mut node: *mut LinkedListNode, mut ref_node: *mut LinkedListNode) {
        interrupt::free(|_| {
            (*node).next = ref_node;
            (*node).prev = (*ref_node).prev;
            (*ref_node).prev = node;
            (*(*node).prev).next = node;
        });
    }

    pub unsafe fn get_size(mut listHead: *mut LinkedListNode) -> usize {
        interrupt::free(|_| {
            let mut size = 0;
            let mut temp: *mut LinkedListNode = core::ptr::null_mut::<LinkedListNode>();

            temp = (*listHead).next;
            while temp != listHead {
                size += 1;
                temp = (*temp).next
            }

            size
        })
    }

    pub unsafe fn get_next_node(mut ref_node: *mut LinkedListNode, mut node: *mut *mut LinkedListNode) {
        interrupt::free(|_| {
            *node = (*ref_node).next;
        });
    }

    pub unsafe fn get_prev_node(mut ref_node: *mut LinkedListNode, mut node: *mut *mut LinkedListNode) {
        interrupt::free(|_| {
            *node = (*ref_node).prev;
        });
    }
}
