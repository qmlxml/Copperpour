### `update_pcb_from_schematic`: a pad-net change is refused only when copper reaches the pad (minor release)

From the `marju212/Konnect` fork (behaviour of commit `7fa1510`, re-implemented on Copperpour's code).

- Before: a pad could not change net while its old or new net had any routed copper anywhere on the
  board, so a part could never leave a busy net such as +5V, and a pad whose track was already removed
  stayed blocked.
- Now, on a live board, the `routed_pad_net_change` conflict is raised when a track, arc or via of
  the pad's **old** net reaches that pad; when a zone pours the old or the new net (unchanged,
  #862); or when a zone's net cannot be read. A custom-shape pad, or a pad whose size cannot be read,
  counts as reached, and so does copper whose geometry could not be read. Layers are not compared,
  so copper on another layer also counts. A copper zone on no net blocks nothing.
- Moving a pad onto a routed net that does not reach it is now allowed; the pad shows a ratsnest to
  its new net.
- Callers that relied on any routed net refusing the change will see fewer conflicts; the message
  and code are unchanged.
