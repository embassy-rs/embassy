# embassy-futures

Utilities for working with futures:

- [`select`](select::select) - waiting for one out of two futures to complete.
- [`select3`](select::select3) - waiting for one out of three futures to complete.
- [`select4`](select::select4) - waiting for one out of four futures to complete.
- [`select_all`](select::select_all) - waiting for one future in a list of futures to complete.
- [`yield_now`](yield_now::yield_now) - yielding the current task.
