;; ari skill ABI v1: WASM skill that calls ari::http_fetch and passes the
;; result straight back as its execute() return value.
;;
;; The hardcoded URL lives in the data segment at offset 256. The bump
;; allocator starts well above it (8192) so the host's ari_alloc-driven
;; response write doesn't clobber the URL or anything else we care about.
(module
  (import "ari" "http_fetch" (func $fetch (param i32 i32) (result i64)))
  (memory (export "memory") 1)

  ;; URL at 256, length 28
  (data (i32.const 256) "https://api.github.com/zen")

  (global $bump (mut i32) (i32.const 8192))

  (func (export "ari_alloc") (param $size i32) (result i32)
    (local $p i32)
    global.get $bump
    local.set $p
    global.get $bump
    local.get $size
    i32.add
    global.set $bump
    local.get $p)

  (func (export "score") (param i32 i32) (result f32)
    f32.const 0.95)

  (func (export "execute") (param i32 i32) (result i64)
    ;; Call http_fetch with the URL at offset 256, length 26.
    ;; Whatever packed pointer it returns becomes our return value verbatim —
    ;; the host will read the JSON it just wrote into our memory back out as
    ;; the response text.
    i32.const 256
    i32.const 26
    call $fetch)
)
