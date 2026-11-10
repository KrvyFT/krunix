    .section .text
    .balign 4
    .globl user_trap_entry
user_trap_entry:
    # sscratch is the kernel stack top in U mode, and zero in S mode.
    csrrw sp, sscratch, sp
    bnez sp, .Lsave_user
    csrrw sp, sscratch, sp
    call handle_kernel_trap

.Lsave_user:
    addi sp, sp, -{frame_size}
    .irp reg, 1,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
        sd x\reg, 8*\reg(sp)
    .endr
    csrrw t0, sscratch, zero
    sd t0, 2*8(sp)
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # app_index stays in the frame while the app is running.
    mv a0, sp
    call handle_user_trap
    j .Lrestore_user

    .globl enter_user
enter_user:
    mv sp, a0

.Lrestore_user:
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    .irp reg, 1,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
        ld x\reg, 8*\reg(sp)
    .endr
    addi sp, sp, {frame_size}
    csrrw sp, sscratch, sp
    sret
