; A ```wf fence in Markdown is handled by `code_fence_block_name`; nothing is
; embedded inside a .wf file itself. Comments get the comment grammar so that
; TODO / FIXME / NOTE are picked out.

((comment) @injection.content
  (#set! injection.language "comment"))
