;; ari skill ABI v1: persistent counter via storage_get / storage_set.
;;
;; On execute:
;;   1. storage_get("counter")
;;   2. if not found → write "1" at offset 4097 and storage_set it
;;   3. otherwise → read existing byte from packed pointer, increment
;;      (wrap '9' → '1'), write new byte at 4097, storage_set
;;   4. return packed (4097 << 32) | 1
;;
;; Key "counter" lives at offset 256, length 7.
(module
  (import "ari" "storage_get" (func $get (param i32 i32) (result i64)))
  (import "ari" "storage_set" (func $set (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 256) "counter")
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

  (func (export "score") (param i32 i32) (result f32) f32.const 0.95)

  (func (export "execute") (param i32 i32) (result i64)
    (local $packed i64)
    (local $existing_ptr i32)
    (local $byte i32)
    (local $new_byte i32)

    ;; storage_get("counter")
    i32.const 256
    i32.const 7
    call $get
    local.set $packed

    local.get $packed
    i64.const 0
    i64.eq
    if (result i64)
      ;; not found: store "1"
      i32.const 4097
      i32.const 49 ;; ASCII '1'
      i32.store8
      i32.const 256 i32.const 7 i32.const 4097 i32.const 1
      call $set
      drop
      i64.const 4097 i64.const 32 i64.shl i64.const 1 i64.or
    else
      ;; found: read existing byte, increment, wrap '9' → '1'
      local.get $packed
      i64.const 32
      i64.shr_u
      i32.wrap_i64
      local.set $existing_ptr
      local.get $existing_ptr
      i32.load8_u
      local.set $byte
      local.get $byte
      i32.const 57 ;; '9'
      i32.ge_s
      if (result i32)
        i32.const 49 ;; '1'
      else
        local.get $byte
        i32.const 1
        i32.add
      end
      local.set $new_byte
      i32.const 4097
      local.get $new_byte
      i32.store8
      i32.const 256 i32.const 7 i32.const 4097 i32.const 1
      call $set
      drop
      i64.const 4097 i64.const 32 i64.shl i64.const 1 i64.or
    end)
)
