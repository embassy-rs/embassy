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

use core::ptr;

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
    pub unsafe fn init_head(mut p_list_head: *mut LinkedListNode) {
        ptr::write_volatile(
            p_list_head,
            LinkedListNode {
                next: p_list_head,
                prev: p_list_head,
            },
        );
    }

    pub unsafe fn is_empty(mut p_list_head: *mut LinkedListNode) -> bool {
        interrupt::free(|_| ptr::read_volatile(p_list_head).next == p_list_head)
    }

    /// Insert `node` after `list_head` and before the next node
    pub unsafe fn insert_head(mut p_list_head: *mut LinkedListNode, mut p_node: *mut LinkedListNode) {
        interrupt::free(|_| {
            let mut list_head = ptr::read_volatile(p_list_head);
            let mut node_next = ptr::read_volatile(list_head.next);
            let node = LinkedListNode {
                next: list_head.next,
                prev: p_list_head,
            };

            list_head.next = p_node;
            node_next.prev = p_node;

            // All nodes must be written because they will all be seen by another core
            ptr::write_volatile(p_node, node);
            ptr::write_volatile(node.next, node_next);
            ptr::write_volatile(p_list_head, list_head);
        });
    }

    /// Insert `node` before `list_tail` and after the second-to-last node
    pub unsafe fn insert_tail(mut p_list_tail: *mut LinkedListNode, mut p_node: *mut LinkedListNode) {
        interrupt::free(|_| {
            let mut list_tail = ptr::read_volatile(p_list_tail);
            let mut node_prev = ptr::read_volatile(list_tail.prev);
            let node = LinkedListNode {
                next: p_list_tail,
                prev: list_tail.prev,
            };

            list_tail.prev = p_node;
            node_prev.next = p_node;

            // All nodes must be written because they will all be seen by another core
            ptr::write_volatile(p_node, node);
            ptr::write_volatile(node.prev, node_prev);
            ptr::write_volatile(p_list_tail, list_tail);
        });
    }

    /// Remove `node` from the linked list
    pub unsafe fn remove_node(mut p_node: *mut LinkedListNode) {
        interrupt::free(|_| {
            let node = ptr::read_volatile(p_node);
            let mut node_prev = ptr::read_volatile(node.prev);
            let mut node_next = ptr::read_volatile(node.next);

            node_prev.next = node.next;
            node_next.prev = node.prev;

            ptr::write_volatile(node.prev, node_prev);
            ptr::write_volatile(node.next, node_next);
        });
    }

    /// Remove `list_head` into `node`
    pub unsafe fn remove_head(mut p_list_head: *mut LinkedListNode, mut p_node: *mut *mut LinkedListNode) {
        interrupt::free(|_| {
            let list_head = ptr::read_volatile(p_list_head);

            // Allowed because a removed node is not seen by another core
            *p_node = list_head.next;
            Self::remove_node(list_head.next);
        });
    }

    /// Remove `list_tail` into `node`
    pub unsafe fn remove_tail(mut p_list_tail: *mut LinkedListNode, mut p_node: *mut *mut LinkedListNode) {
        interrupt::free(|_| {
            let list_tail = ptr::read_volatile(p_list_tail);

            // Allowed because a removed node is not seen by another core
            *p_node = list_tail.prev;
            Self::remove_node(list_tail.prev);
        });
    }

    // TODO: convert these other functions to volatile semantics

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

    pub unsafe fn get_size(mut list_head: *mut LinkedListNode) -> usize {
        interrupt::free(|_| {
            let mut size = 0;
            let mut temp: *mut LinkedListNode = core::ptr::null_mut::<LinkedListNode>();

            temp = (*list_head).next;
            while temp != list_head {
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
