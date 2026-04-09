;; ari skill ABI v1 reference module
;;
;; Exports the four required symbols and returns a fixed greeting from
;; inside its own linear memory. No host imports beyond what's allowed.
(module
  (memory (export "memory") 1)
  (data (i32.const 2048) "wasm hello")

  ;; Bump allocator. Starts at offset 1024 (anything below the data segment
  ;; at 2048 is fair game for the host's input string).
  (global $bump (mut i32) (i32.const 1024))

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
    ;; pack (2048 << 32) | 10
    i64.const 2048
    i64.const 32
    i64.shl
    i64.const 10
    i64.or)
)
