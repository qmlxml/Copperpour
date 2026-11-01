### `update_pcb_from_schematic`: a pad-net change is refused only when copper reaches the pad (minor release)

From the `marju212/Konnect` fork (behaviour of commit `7fa1510`, re-implemented on Copperpour's code).

- Before: a pad could not change net while its old or new net had any routed copper anywhere on the
  board, so a part could never leave a busy net such as +5V, and a pad whose track was already removed
  stayed blocked.
- Now, on a live board, the `routed_pad_net_change` conflict is raised when a track, arc or via of
  the pad's **old** net reaches that pad, or when a zone's net cannot be read. (A zone pouring the
  old or new net also refused the change in this note's first form; that was replaced in the same
  release by `sync-pour-no-longer-blocks-pad-net-change.md`: a pour no longer blocks, and the
  response names it in `zones_to_refill`.) A custom-shape pad, or a pad whose size cannot be read,
  counts as reached, and so does copper whose geometry could not be read. Layers are not compared,
  so copper on another layer also counts. A copper zone on no net blocks nothing.
- Moving a pad onto a routed net that does not reach it is now allowed; the pad shows a ratsnest to
  its new net.
- Callers that relied on any routed net refusing the change will see fewer conflicts. The code is
  unchanged; the message now ends with the cause instead of "while routed copper uses that net": the
  track, arc or via that reaches the pad (with its UUID and position), an unreadable zone, or copper
  whose geometry could not be read, with the tools that remove it.
