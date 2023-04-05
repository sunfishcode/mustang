	.text
	.file	"aarch64_outline_atomics.s"
	.section	.text.__aarch64_ldadd1_acq,"ax",@progbits
	.globl	__aarch64_ldadd1_acq
	.p2align	2
	.type	__aarch64_ldadd1_acq,@function
__aarch64_ldadd1_acq:
	.cfi_startproc
.LBB0_1:
	ldaxrb	w8, [x1]
	add	w9, w8, w0
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB0_1
	mov	w0, w8
	ret
.Lfunc_end0:
	.size	__aarch64_ldadd1_acq, .Lfunc_end0-__aarch64_ldadd1_acq
	.cfi_endproc

	.section	.text.__aarch64_ldadd1_acq_rel,"ax",@progbits
	.globl	__aarch64_ldadd1_acq_rel
	.p2align	2
	.type	__aarch64_ldadd1_acq_rel,@function
__aarch64_ldadd1_acq_rel:
	.cfi_startproc
.LBB1_1:
	ldaxrb	w8, [x1]
	add	w9, w8, w0
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB1_1
	mov	w0, w8
	ret
.Lfunc_end1:
	.size	__aarch64_ldadd1_acq_rel, .Lfunc_end1-__aarch64_ldadd1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd1_rel,"ax",@progbits
	.globl	__aarch64_ldadd1_rel
	.p2align	2
	.type	__aarch64_ldadd1_rel,@function
__aarch64_ldadd1_rel:
	.cfi_startproc
.LBB2_1:
	ldxrb	w8, [x1]
	add	w9, w8, w0
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB2_1
	mov	w0, w8
	ret
.Lfunc_end2:
	.size	__aarch64_ldadd1_rel, .Lfunc_end2-__aarch64_ldadd1_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd1_relax,"ax",@progbits
	.globl	__aarch64_ldadd1_relax
	.p2align	2
	.type	__aarch64_ldadd1_relax,@function
__aarch64_ldadd1_relax:
	.cfi_startproc
.LBB3_1:
	ldxrb	w8, [x1]
	add	w9, w8, w0
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB3_1
	mov	w0, w8
	ret
.Lfunc_end3:
	.size	__aarch64_ldadd1_relax, .Lfunc_end3-__aarch64_ldadd1_relax
	.cfi_endproc

	.section	.text.__aarch64_ldadd2_acq,"ax",@progbits
	.globl	__aarch64_ldadd2_acq
	.p2align	2
	.type	__aarch64_ldadd2_acq,@function
__aarch64_ldadd2_acq:
	.cfi_startproc
.LBB4_1:
	ldaxrh	w8, [x1]
	add	w9, w8, w0
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB4_1
	mov	w0, w8
	ret
.Lfunc_end4:
	.size	__aarch64_ldadd2_acq, .Lfunc_end4-__aarch64_ldadd2_acq
	.cfi_endproc

	.section	.text.__aarch64_ldadd2_acq_rel,"ax",@progbits
	.globl	__aarch64_ldadd2_acq_rel
	.p2align	2
	.type	__aarch64_ldadd2_acq_rel,@function
__aarch64_ldadd2_acq_rel:
	.cfi_startproc
.LBB5_1:
	ldaxrh	w8, [x1]
	add	w9, w8, w0
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB5_1
	mov	w0, w8
	ret
.Lfunc_end5:
	.size	__aarch64_ldadd2_acq_rel, .Lfunc_end5-__aarch64_ldadd2_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd2_rel,"ax",@progbits
	.globl	__aarch64_ldadd2_rel
	.p2align	2
	.type	__aarch64_ldadd2_rel,@function
__aarch64_ldadd2_rel:
	.cfi_startproc
.LBB6_1:
	ldxrh	w8, [x1]
	add	w9, w8, w0
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB6_1
	mov	w0, w8
	ret
.Lfunc_end6:
	.size	__aarch64_ldadd2_rel, .Lfunc_end6-__aarch64_ldadd2_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd2_relax,"ax",@progbits
	.globl	__aarch64_ldadd2_relax
	.p2align	2
	.type	__aarch64_ldadd2_relax,@function
__aarch64_ldadd2_relax:
	.cfi_startproc
.LBB7_1:
	ldxrh	w8, [x1]
	add	w9, w8, w0
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB7_1
	mov	w0, w8
	ret
.Lfunc_end7:
	.size	__aarch64_ldadd2_relax, .Lfunc_end7-__aarch64_ldadd2_relax
	.cfi_endproc

	.section	.text.__aarch64_ldadd4_acq,"ax",@progbits
	.globl	__aarch64_ldadd4_acq
	.p2align	2
	.type	__aarch64_ldadd4_acq,@function
__aarch64_ldadd4_acq:
	.cfi_startproc
.LBB8_1:
	ldaxr	w8, [x1]
	add	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB8_1
	mov	w0, w8
	ret
.Lfunc_end8:
	.size	__aarch64_ldadd4_acq, .Lfunc_end8-__aarch64_ldadd4_acq
	.cfi_endproc

	.section	.text.__aarch64_ldadd4_acq_rel,"ax",@progbits
	.globl	__aarch64_ldadd4_acq_rel
	.p2align	2
	.type	__aarch64_ldadd4_acq_rel,@function
__aarch64_ldadd4_acq_rel:
	.cfi_startproc
.LBB9_1:
	ldaxr	w8, [x1]
	add	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB9_1
	mov	w0, w8
	ret
.Lfunc_end9:
	.size	__aarch64_ldadd4_acq_rel, .Lfunc_end9-__aarch64_ldadd4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd4_rel,"ax",@progbits
	.globl	__aarch64_ldadd4_rel
	.p2align	2
	.type	__aarch64_ldadd4_rel,@function
__aarch64_ldadd4_rel:
	.cfi_startproc
.LBB10_1:
	ldxr	w8, [x1]
	add	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB10_1
	mov	w0, w8
	ret
.Lfunc_end10:
	.size	__aarch64_ldadd4_rel, .Lfunc_end10-__aarch64_ldadd4_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd4_relax,"ax",@progbits
	.globl	__aarch64_ldadd4_relax
	.p2align	2
	.type	__aarch64_ldadd4_relax,@function
__aarch64_ldadd4_relax:
	.cfi_startproc
.LBB11_1:
	ldxr	w8, [x1]
	add	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB11_1
	mov	w0, w8
	ret
.Lfunc_end11:
	.size	__aarch64_ldadd4_relax, .Lfunc_end11-__aarch64_ldadd4_relax
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_acq,"ax",@progbits
	.globl	__aarch64_ldadd8_acq
	.p2align	2
	.type	__aarch64_ldadd8_acq,@function
__aarch64_ldadd8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB12_1:
	ldaxr	x0, [x1]
	add	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB12_1
	ret
.Lfunc_end12:
	.size	__aarch64_ldadd8_acq, .Lfunc_end12-__aarch64_ldadd8_acq
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_acq_rel,"ax",@progbits
	.globl	__aarch64_ldadd8_acq_rel
	.p2align	2
	.type	__aarch64_ldadd8_acq_rel,@function
__aarch64_ldadd8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB13_1:
	ldaxr	x0, [x1]
	add	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB13_1
	ret
.Lfunc_end13:
	.size	__aarch64_ldadd8_acq_rel, .Lfunc_end13-__aarch64_ldadd8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_rel,"ax",@progbits
	.globl	__aarch64_ldadd8_rel
	.p2align	2
	.type	__aarch64_ldadd8_rel,@function
__aarch64_ldadd8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB14_1:
	ldxr	x0, [x1]
	add	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB14_1
	ret
.Lfunc_end14:
	.size	__aarch64_ldadd8_rel, .Lfunc_end14-__aarch64_ldadd8_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_relax,"ax",@progbits
	.globl	__aarch64_ldadd8_relax
	.p2align	2
	.type	__aarch64_ldadd8_relax,@function
__aarch64_ldadd8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB15_1:
	ldxr	x0, [x1]
	add	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB15_1
	ret
.Lfunc_end15:
	.size	__aarch64_ldadd8_relax, .Lfunc_end15-__aarch64_ldadd8_relax
	.cfi_endproc

	.section	.text.__aarch64_ldeor1_acq,"ax",@progbits
	.globl	__aarch64_ldeor1_acq
	.p2align	2
	.type	__aarch64_ldeor1_acq,@function
__aarch64_ldeor1_acq:
	.cfi_startproc
.LBB16_1:
	ldaxrb	w8, [x1]
	eor	w9, w8, w0
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB16_1
	mov	w0, w8
	ret
.Lfunc_end16:
	.size	__aarch64_ldeor1_acq, .Lfunc_end16-__aarch64_ldeor1_acq
	.cfi_endproc

	.section	.text.__aarch64_ldeor1_acq_rel,"ax",@progbits
	.globl	__aarch64_ldeor1_acq_rel
	.p2align	2
	.type	__aarch64_ldeor1_acq_rel,@function
__aarch64_ldeor1_acq_rel:
	.cfi_startproc
.LBB17_1:
	ldaxrb	w8, [x1]
	eor	w9, w8, w0
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB17_1
	mov	w0, w8
	ret
.Lfunc_end17:
	.size	__aarch64_ldeor1_acq_rel, .Lfunc_end17-__aarch64_ldeor1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor1_rel,"ax",@progbits
	.globl	__aarch64_ldeor1_rel
	.p2align	2
	.type	__aarch64_ldeor1_rel,@function
__aarch64_ldeor1_rel:
	.cfi_startproc
.LBB18_1:
	ldxrb	w8, [x1]
	eor	w9, w8, w0
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB18_1
	mov	w0, w8
	ret
.Lfunc_end18:
	.size	__aarch64_ldeor1_rel, .Lfunc_end18-__aarch64_ldeor1_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor1_relax,"ax",@progbits
	.globl	__aarch64_ldeor1_relax
	.p2align	2
	.type	__aarch64_ldeor1_relax,@function
__aarch64_ldeor1_relax:
	.cfi_startproc
.LBB19_1:
	ldxrb	w8, [x1]
	eor	w9, w8, w0
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB19_1
	mov	w0, w8
	ret
.Lfunc_end19:
	.size	__aarch64_ldeor1_relax, .Lfunc_end19-__aarch64_ldeor1_relax
	.cfi_endproc

	.section	.text.__aarch64_ldeor2_acq,"ax",@progbits
	.globl	__aarch64_ldeor2_acq
	.p2align	2
	.type	__aarch64_ldeor2_acq,@function
__aarch64_ldeor2_acq:
	.cfi_startproc
.LBB20_1:
	ldaxrh	w8, [x1]
	eor	w9, w8, w0
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB20_1
	mov	w0, w8
	ret
.Lfunc_end20:
	.size	__aarch64_ldeor2_acq, .Lfunc_end20-__aarch64_ldeor2_acq
	.cfi_endproc

	.section	.text.__aarch64_ldeor2_acq_rel,"ax",@progbits
	.globl	__aarch64_ldeor2_acq_rel
	.p2align	2
	.type	__aarch64_ldeor2_acq_rel,@function
__aarch64_ldeor2_acq_rel:
	.cfi_startproc
.LBB21_1:
	ldaxrh	w8, [x1]
	eor	w9, w8, w0
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB21_1
	mov	w0, w8
	ret
.Lfunc_end21:
	.size	__aarch64_ldeor2_acq_rel, .Lfunc_end21-__aarch64_ldeor2_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor2_rel,"ax",@progbits
	.globl	__aarch64_ldeor2_rel
	.p2align	2
	.type	__aarch64_ldeor2_rel,@function
__aarch64_ldeor2_rel:
	.cfi_startproc
.LBB22_1:
	ldxrh	w8, [x1]
	eor	w9, w8, w0
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB22_1
	mov	w0, w8
	ret
.Lfunc_end22:
	.size	__aarch64_ldeor2_rel, .Lfunc_end22-__aarch64_ldeor2_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor2_relax,"ax",@progbits
	.globl	__aarch64_ldeor2_relax
	.p2align	2
	.type	__aarch64_ldeor2_relax,@function
__aarch64_ldeor2_relax:
	.cfi_startproc
.LBB23_1:
	ldxrh	w8, [x1]
	eor	w9, w8, w0
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB23_1
	mov	w0, w8
	ret
.Lfunc_end23:
	.size	__aarch64_ldeor2_relax, .Lfunc_end23-__aarch64_ldeor2_relax
	.cfi_endproc

	.section	.text.__aarch64_ldeor4_acq,"ax",@progbits
	.globl	__aarch64_ldeor4_acq
	.p2align	2
	.type	__aarch64_ldeor4_acq,@function
__aarch64_ldeor4_acq:
	.cfi_startproc
.LBB24_1:
	ldaxr	w8, [x1]
	eor	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB24_1
	mov	w0, w8
	ret
.Lfunc_end24:
	.size	__aarch64_ldeor4_acq, .Lfunc_end24-__aarch64_ldeor4_acq
	.cfi_endproc

	.section	.text.__aarch64_ldeor4_acq_rel,"ax",@progbits
	.globl	__aarch64_ldeor4_acq_rel
	.p2align	2
	.type	__aarch64_ldeor4_acq_rel,@function
__aarch64_ldeor4_acq_rel:
	.cfi_startproc
.LBB25_1:
	ldaxr	w8, [x1]
	eor	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB25_1
	mov	w0, w8
	ret
.Lfunc_end25:
	.size	__aarch64_ldeor4_acq_rel, .Lfunc_end25-__aarch64_ldeor4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor4_rel,"ax",@progbits
	.globl	__aarch64_ldeor4_rel
	.p2align	2
	.type	__aarch64_ldeor4_rel,@function
__aarch64_ldeor4_rel:
	.cfi_startproc
.LBB26_1:
	ldxr	w8, [x1]
	eor	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB26_1
	mov	w0, w8
	ret
.Lfunc_end26:
	.size	__aarch64_ldeor4_rel, .Lfunc_end26-__aarch64_ldeor4_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor4_relax,"ax",@progbits
	.globl	__aarch64_ldeor4_relax
	.p2align	2
	.type	__aarch64_ldeor4_relax,@function
__aarch64_ldeor4_relax:
	.cfi_startproc
.LBB27_1:
	ldxr	w8, [x1]
	eor	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB27_1
	mov	w0, w8
	ret
.Lfunc_end27:
	.size	__aarch64_ldeor4_relax, .Lfunc_end27-__aarch64_ldeor4_relax
	.cfi_endproc

	.section	.text.__aarch64_ldeor8_acq,"ax",@progbits
	.globl	__aarch64_ldeor8_acq
	.p2align	2
	.type	__aarch64_ldeor8_acq,@function
__aarch64_ldeor8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB28_1:
	ldaxr	x0, [x1]
	eor	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB28_1
	ret
.Lfunc_end28:
	.size	__aarch64_ldeor8_acq, .Lfunc_end28-__aarch64_ldeor8_acq
	.cfi_endproc

	.section	.text.__aarch64_ldeor8_acq_rel,"ax",@progbits
	.globl	__aarch64_ldeor8_acq_rel
	.p2align	2
	.type	__aarch64_ldeor8_acq_rel,@function
__aarch64_ldeor8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB29_1:
	ldaxr	x0, [x1]
	eor	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB29_1
	ret
.Lfunc_end29:
	.size	__aarch64_ldeor8_acq_rel, .Lfunc_end29-__aarch64_ldeor8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor8_rel,"ax",@progbits
	.globl	__aarch64_ldeor8_rel
	.p2align	2
	.type	__aarch64_ldeor8_rel,@function
__aarch64_ldeor8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB30_1:
	ldxr	x0, [x1]
	eor	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB30_1
	ret
.Lfunc_end30:
	.size	__aarch64_ldeor8_rel, .Lfunc_end30-__aarch64_ldeor8_rel
	.cfi_endproc

	.section	.text.__aarch64_ldeor8_relax,"ax",@progbits
	.globl	__aarch64_ldeor8_relax
	.p2align	2
	.type	__aarch64_ldeor8_relax,@function
__aarch64_ldeor8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB31_1:
	ldxr	x0, [x1]
	eor	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB31_1
	ret
.Lfunc_end31:
	.size	__aarch64_ldeor8_relax, .Lfunc_end31-__aarch64_ldeor8_relax
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_acq,"ax",@progbits
	.globl	__aarch64_ldclr1_acq
	.p2align	2
	.type	__aarch64_ldclr1_acq,@function
__aarch64_ldclr1_acq:
	.cfi_startproc
	mvn	w8, w0
.LBB32_1:
	ldaxrb	w0, [x1]
	and	w9, w0, w8
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB32_1
	ret
.Lfunc_end32:
	.size	__aarch64_ldclr1_acq, .Lfunc_end32-__aarch64_ldclr1_acq
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_acq_rel,"ax",@progbits
	.globl	__aarch64_ldclr1_acq_rel
	.p2align	2
	.type	__aarch64_ldclr1_acq_rel,@function
__aarch64_ldclr1_acq_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB33_1:
	ldaxrb	w0, [x1]
	and	w9, w0, w8
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB33_1
	ret
.Lfunc_end33:
	.size	__aarch64_ldclr1_acq_rel, .Lfunc_end33-__aarch64_ldclr1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_rel,"ax",@progbits
	.globl	__aarch64_ldclr1_rel
	.p2align	2
	.type	__aarch64_ldclr1_rel,@function
__aarch64_ldclr1_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB34_1:
	ldxrb	w0, [x1]
	and	w9, w0, w8
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB34_1
	ret
.Lfunc_end34:
	.size	__aarch64_ldclr1_rel, .Lfunc_end34-__aarch64_ldclr1_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_relax,"ax",@progbits
	.globl	__aarch64_ldclr1_relax
	.p2align	2
	.type	__aarch64_ldclr1_relax,@function
__aarch64_ldclr1_relax:
	.cfi_startproc
	mvn	w8, w0
.LBB35_1:
	ldxrb	w0, [x1]
	and	w9, w0, w8
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB35_1
	ret
.Lfunc_end35:
	.size	__aarch64_ldclr1_relax, .Lfunc_end35-__aarch64_ldclr1_relax
	.cfi_endproc

	.section	.text.__aarch64_ldclr2_acq,"ax",@progbits
	.globl	__aarch64_ldclr2_acq
	.p2align	2
	.type	__aarch64_ldclr2_acq,@function
__aarch64_ldclr2_acq:
	.cfi_startproc
	mvn	w8, w0
.LBB36_1:
	ldaxrh	w0, [x1]
	and	w9, w0, w8
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB36_1
	ret
.Lfunc_end36:
	.size	__aarch64_ldclr2_acq, .Lfunc_end36-__aarch64_ldclr2_acq
	.cfi_endproc

	.section	.text.__aarch64_ldclr2_acq_rel,"ax",@progbits
	.globl	__aarch64_ldclr2_acq_rel
	.p2align	2
	.type	__aarch64_ldclr2_acq_rel,@function
__aarch64_ldclr2_acq_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB37_1:
	ldaxrh	w0, [x1]
	and	w9, w0, w8
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB37_1
	ret
.Lfunc_end37:
	.size	__aarch64_ldclr2_acq_rel, .Lfunc_end37-__aarch64_ldclr2_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr2_rel,"ax",@progbits
	.globl	__aarch64_ldclr2_rel
	.p2align	2
	.type	__aarch64_ldclr2_rel,@function
__aarch64_ldclr2_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB38_1:
	ldxrh	w0, [x1]
	and	w9, w0, w8
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB38_1
	ret
.Lfunc_end38:
	.size	__aarch64_ldclr2_rel, .Lfunc_end38-__aarch64_ldclr2_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr2_relax,"ax",@progbits
	.globl	__aarch64_ldclr2_relax
	.p2align	2
	.type	__aarch64_ldclr2_relax,@function
__aarch64_ldclr2_relax:
	.cfi_startproc
	mvn	w8, w0
.LBB39_1:
	ldxrh	w0, [x1]
	and	w9, w0, w8
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB39_1
	ret
.Lfunc_end39:
	.size	__aarch64_ldclr2_relax, .Lfunc_end39-__aarch64_ldclr2_relax
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_acq,"ax",@progbits
	.globl	__aarch64_ldclr4_acq
	.p2align	2
	.type	__aarch64_ldclr4_acq,@function
__aarch64_ldclr4_acq:
	.cfi_startproc
	mvn	w8, w0
.LBB40_1:
	ldaxr	w0, [x1]
	and	w9, w0, w8
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB40_1
	ret
.Lfunc_end40:
	.size	__aarch64_ldclr4_acq, .Lfunc_end40-__aarch64_ldclr4_acq
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_acq_rel,"ax",@progbits
	.globl	__aarch64_ldclr4_acq_rel
	.p2align	2
	.type	__aarch64_ldclr4_acq_rel,@function
__aarch64_ldclr4_acq_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB41_1:
	ldaxr	w0, [x1]
	and	w9, w0, w8
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB41_1
	ret
.Lfunc_end41:
	.size	__aarch64_ldclr4_acq_rel, .Lfunc_end41-__aarch64_ldclr4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_rel,"ax",@progbits
	.globl	__aarch64_ldclr4_rel
	.p2align	2
	.type	__aarch64_ldclr4_rel,@function
__aarch64_ldclr4_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB42_1:
	ldxr	w0, [x1]
	and	w9, w0, w8
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB42_1
	ret
.Lfunc_end42:
	.size	__aarch64_ldclr4_rel, .Lfunc_end42-__aarch64_ldclr4_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_relax,"ax",@progbits
	.globl	__aarch64_ldclr4_relax
	.p2align	2
	.type	__aarch64_ldclr4_relax,@function
__aarch64_ldclr4_relax:
	.cfi_startproc
	mvn	w8, w0
.LBB43_1:
	ldxr	w0, [x1]
	and	w9, w0, w8
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB43_1
	ret
.Lfunc_end43:
	.size	__aarch64_ldclr4_relax, .Lfunc_end43-__aarch64_ldclr4_relax
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_acq,"ax",@progbits
	.globl	__aarch64_ldclr8_acq
	.p2align	2
	.type	__aarch64_ldclr8_acq,@function
__aarch64_ldclr8_acq:
	.cfi_startproc
	mvn	x8, x0
.LBB44_1:
	ldaxr	x0, [x1]
	and	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB44_1
	ret
.Lfunc_end44:
	.size	__aarch64_ldclr8_acq, .Lfunc_end44-__aarch64_ldclr8_acq
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_acq_rel,"ax",@progbits
	.globl	__aarch64_ldclr8_acq_rel
	.p2align	2
	.type	__aarch64_ldclr8_acq_rel,@function
__aarch64_ldclr8_acq_rel:
	.cfi_startproc
	mvn	x8, x0
.LBB45_1:
	ldaxr	x0, [x1]
	and	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB45_1
	ret
.Lfunc_end45:
	.size	__aarch64_ldclr8_acq_rel, .Lfunc_end45-__aarch64_ldclr8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_rel,"ax",@progbits
	.globl	__aarch64_ldclr8_rel
	.p2align	2
	.type	__aarch64_ldclr8_rel,@function
__aarch64_ldclr8_rel:
	.cfi_startproc
	mvn	x8, x0
.LBB46_1:
	ldxr	x0, [x1]
	and	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB46_1
	ret
.Lfunc_end46:
	.size	__aarch64_ldclr8_rel, .Lfunc_end46-__aarch64_ldclr8_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_relax,"ax",@progbits
	.globl	__aarch64_ldclr8_relax
	.p2align	2
	.type	__aarch64_ldclr8_relax,@function
__aarch64_ldclr8_relax:
	.cfi_startproc
	mvn	x8, x0
.LBB47_1:
	ldxr	x0, [x1]
	and	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB47_1
	ret
.Lfunc_end47:
	.size	__aarch64_ldclr8_relax, .Lfunc_end47-__aarch64_ldclr8_relax
	.cfi_endproc

	.section	.text.__aarch64_swp1_acq,"ax",@progbits
	.globl	__aarch64_swp1_acq
	.p2align	2
	.type	__aarch64_swp1_acq,@function
__aarch64_swp1_acq:
	.cfi_startproc
	mov	w8, w0
.LBB48_1:
	ldaxrb	w0, [x1]
	stxrb	w9, w8, [x1]
	cbnz	w9, .LBB48_1
	ret
.Lfunc_end48:
	.size	__aarch64_swp1_acq, .Lfunc_end48-__aarch64_swp1_acq
	.cfi_endproc

	.section	.text.__aarch64_swp1_acq_rel,"ax",@progbits
	.globl	__aarch64_swp1_acq_rel
	.p2align	2
	.type	__aarch64_swp1_acq_rel,@function
__aarch64_swp1_acq_rel:
	.cfi_startproc
	mov	w8, w0
.LBB49_1:
	ldaxrb	w0, [x1]
	stlxrb	w9, w8, [x1]
	cbnz	w9, .LBB49_1
	ret
.Lfunc_end49:
	.size	__aarch64_swp1_acq_rel, .Lfunc_end49-__aarch64_swp1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_swp1_rel,"ax",@progbits
	.globl	__aarch64_swp1_rel
	.p2align	2
	.type	__aarch64_swp1_rel,@function
__aarch64_swp1_rel:
	.cfi_startproc
	mov	w8, w0
.LBB50_1:
	ldxrb	w0, [x1]
	stlxrb	w9, w8, [x1]
	cbnz	w9, .LBB50_1
	ret
.Lfunc_end50:
	.size	__aarch64_swp1_rel, .Lfunc_end50-__aarch64_swp1_rel
	.cfi_endproc

	.section	.text.__aarch64_swp1_relax,"ax",@progbits
	.globl	__aarch64_swp1_relax
	.p2align	2
	.type	__aarch64_swp1_relax,@function
__aarch64_swp1_relax:
	.cfi_startproc
	mov	w8, w0
.LBB51_1:
	ldxrb	w0, [x1]
	stxrb	w9, w8, [x1]
	cbnz	w9, .LBB51_1
	ret
.Lfunc_end51:
	.size	__aarch64_swp1_relax, .Lfunc_end51-__aarch64_swp1_relax
	.cfi_endproc

	.section	.text.__aarch64_swp2_acq,"ax",@progbits
	.globl	__aarch64_swp2_acq
	.p2align	2
	.type	__aarch64_swp2_acq,@function
__aarch64_swp2_acq:
	.cfi_startproc
	mov	w8, w0
.LBB52_1:
	ldaxrh	w0, [x1]
	stxrh	w9, w8, [x1]
	cbnz	w9, .LBB52_1
	ret
.Lfunc_end52:
	.size	__aarch64_swp2_acq, .Lfunc_end52-__aarch64_swp2_acq
	.cfi_endproc

	.section	.text.__aarch64_swp2_acq_rel,"ax",@progbits
	.globl	__aarch64_swp2_acq_rel
	.p2align	2
	.type	__aarch64_swp2_acq_rel,@function
__aarch64_swp2_acq_rel:
	.cfi_startproc
	mov	w8, w0
.LBB53_1:
	ldaxrh	w0, [x1]
	stlxrh	w9, w8, [x1]
	cbnz	w9, .LBB53_1
	ret
.Lfunc_end53:
	.size	__aarch64_swp2_acq_rel, .Lfunc_end53-__aarch64_swp2_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_swp2_rel,"ax",@progbits
	.globl	__aarch64_swp2_rel
	.p2align	2
	.type	__aarch64_swp2_rel,@function
__aarch64_swp2_rel:
	.cfi_startproc
	mov	w8, w0
.LBB54_1:
	ldxrh	w0, [x1]
	stlxrh	w9, w8, [x1]
	cbnz	w9, .LBB54_1
	ret
.Lfunc_end54:
	.size	__aarch64_swp2_rel, .Lfunc_end54-__aarch64_swp2_rel
	.cfi_endproc

	.section	.text.__aarch64_swp2_relax,"ax",@progbits
	.globl	__aarch64_swp2_relax
	.p2align	2
	.type	__aarch64_swp2_relax,@function
__aarch64_swp2_relax:
	.cfi_startproc
	mov	w8, w0
.LBB55_1:
	ldxrh	w0, [x1]
	stxrh	w9, w8, [x1]
	cbnz	w9, .LBB55_1
	ret
.Lfunc_end55:
	.size	__aarch64_swp2_relax, .Lfunc_end55-__aarch64_swp2_relax
	.cfi_endproc

	.section	.text.__aarch64_swp4_acq,"ax",@progbits
	.globl	__aarch64_swp4_acq
	.p2align	2
	.type	__aarch64_swp4_acq,@function
__aarch64_swp4_acq:
	.cfi_startproc
	mov	w8, w0
.LBB56_1:
	ldaxr	w0, [x1]
	stxr	w9, w8, [x1]
	cbnz	w9, .LBB56_1
	ret
.Lfunc_end56:
	.size	__aarch64_swp4_acq, .Lfunc_end56-__aarch64_swp4_acq
	.cfi_endproc

	.section	.text.__aarch64_swp4_acq_rel,"ax",@progbits
	.globl	__aarch64_swp4_acq_rel
	.p2align	2
	.type	__aarch64_swp4_acq_rel,@function
__aarch64_swp4_acq_rel:
	.cfi_startproc
	mov	w8, w0
.LBB57_1:
	ldaxr	w0, [x1]
	stlxr	w9, w8, [x1]
	cbnz	w9, .LBB57_1
	ret
.Lfunc_end57:
	.size	__aarch64_swp4_acq_rel, .Lfunc_end57-__aarch64_swp4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_swp4_rel,"ax",@progbits
	.globl	__aarch64_swp4_rel
	.p2align	2
	.type	__aarch64_swp4_rel,@function
__aarch64_swp4_rel:
	.cfi_startproc
	mov	w8, w0
.LBB58_1:
	ldxr	w0, [x1]
	stlxr	w9, w8, [x1]
	cbnz	w9, .LBB58_1
	ret
.Lfunc_end58:
	.size	__aarch64_swp4_rel, .Lfunc_end58-__aarch64_swp4_rel
	.cfi_endproc

	.section	.text.__aarch64_swp4_relax,"ax",@progbits
	.globl	__aarch64_swp4_relax
	.p2align	2
	.type	__aarch64_swp4_relax,@function
__aarch64_swp4_relax:
	.cfi_startproc
	mov	w8, w0
.LBB59_1:
	ldxr	w0, [x1]
	stxr	w9, w8, [x1]
	cbnz	w9, .LBB59_1
	ret
.Lfunc_end59:
	.size	__aarch64_swp4_relax, .Lfunc_end59-__aarch64_swp4_relax
	.cfi_endproc

	.section	.text.__aarch64_swp8_acq,"ax",@progbits
	.globl	__aarch64_swp8_acq
	.p2align	2
	.type	__aarch64_swp8_acq,@function
__aarch64_swp8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB60_1:
	ldaxr	x0, [x1]
	stxr	w9, x8, [x1]
	cbnz	w9, .LBB60_1
	ret
.Lfunc_end60:
	.size	__aarch64_swp8_acq, .Lfunc_end60-__aarch64_swp8_acq
	.cfi_endproc

	.section	.text.__aarch64_swp8_acq_rel,"ax",@progbits
	.globl	__aarch64_swp8_acq_rel
	.p2align	2
	.type	__aarch64_swp8_acq_rel,@function
__aarch64_swp8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB61_1:
	ldaxr	x0, [x1]
	stlxr	w9, x8, [x1]
	cbnz	w9, .LBB61_1
	ret
.Lfunc_end61:
	.size	__aarch64_swp8_acq_rel, .Lfunc_end61-__aarch64_swp8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_swp8_rel,"ax",@progbits
	.globl	__aarch64_swp8_rel
	.p2align	2
	.type	__aarch64_swp8_rel,@function
__aarch64_swp8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB62_1:
	ldxr	x0, [x1]
	stlxr	w9, x8, [x1]
	cbnz	w9, .LBB62_1
	ret
.Lfunc_end62:
	.size	__aarch64_swp8_rel, .Lfunc_end62-__aarch64_swp8_rel
	.cfi_endproc

	.section	.text.__aarch64_swp8_relax,"ax",@progbits
	.globl	__aarch64_swp8_relax
	.p2align	2
	.type	__aarch64_swp8_relax,@function
__aarch64_swp8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB63_1:
	ldxr	x0, [x1]
	stxr	w9, x8, [x1]
	cbnz	w9, .LBB63_1
	ret
.Lfunc_end63:
	.size	__aarch64_swp8_relax, .Lfunc_end63-__aarch64_swp8_relax
	.cfi_endproc

	.section	.text.__aarch64_cas1_acq,"ax",@progbits
	.globl	__aarch64_cas1_acq
	.p2align	2
	.type	__aarch64_cas1_acq,@function
__aarch64_cas1_acq:
	.cfi_startproc
	mov	w9, w1
.LBB64_1:
	ldaxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB64_4
	stxrb	w10, w9, [x2]
	cbnz	w10, .LBB64_1
	mov	w0, w8
	ret
.LBB64_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end64:
	.size	__aarch64_cas1_acq, .Lfunc_end64-__aarch64_cas1_acq
	.cfi_endproc

	.section	.text.__aarch64_cas1_acq_rel,"ax",@progbits
	.globl	__aarch64_cas1_acq_rel
	.p2align	2
	.type	__aarch64_cas1_acq_rel,@function
__aarch64_cas1_acq_rel:
	.cfi_startproc
	mov	w9, w1
.LBB65_1:
	ldaxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB65_4
	stlxrb	w10, w9, [x2]
	cbnz	w10, .LBB65_1
	mov	w0, w8
	ret
.LBB65_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end65:
	.size	__aarch64_cas1_acq_rel, .Lfunc_end65-__aarch64_cas1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_cas1_rel,"ax",@progbits
	.globl	__aarch64_cas1_rel
	.p2align	2
	.type	__aarch64_cas1_rel,@function
__aarch64_cas1_rel:
	.cfi_startproc
	mov	w9, w1
.LBB66_1:
	ldxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB66_4
	stlxrb	w10, w9, [x2]
	cbnz	w10, .LBB66_1
	mov	w0, w8
	ret
.LBB66_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end66:
	.size	__aarch64_cas1_rel, .Lfunc_end66-__aarch64_cas1_rel
	.cfi_endproc

	.section	.text.__aarch64_cas1_relax,"ax",@progbits
	.globl	__aarch64_cas1_relax
	.p2align	2
	.type	__aarch64_cas1_relax,@function
__aarch64_cas1_relax:
	.cfi_startproc
	mov	w9, w1
.LBB67_1:
	ldxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB67_4
	stxrb	w10, w9, [x2]
	cbnz	w10, .LBB67_1
	mov	w0, w8
	ret
.LBB67_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end67:
	.size	__aarch64_cas1_relax, .Lfunc_end67-__aarch64_cas1_relax
	.cfi_endproc

	.section	.text.__aarch64_cas2_acq,"ax",@progbits
	.globl	__aarch64_cas2_acq
	.p2align	2
	.type	__aarch64_cas2_acq,@function
__aarch64_cas2_acq:
	.cfi_startproc
	mov	w9, w1
.LBB68_1:
	ldaxrh	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB68_4
	stxrh	w10, w9, [x2]
	cbnz	w10, .LBB68_1
	mov	w0, w8
	ret
.LBB68_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end68:
	.size	__aarch64_cas2_acq, .Lfunc_end68-__aarch64_cas2_acq
	.cfi_endproc

	.section	.text.__aarch64_cas2_acq_rel,"ax",@progbits
	.globl	__aarch64_cas2_acq_rel
	.p2align	2
	.type	__aarch64_cas2_acq_rel,@function
__aarch64_cas2_acq_rel:
	.cfi_startproc
	mov	w9, w1
.LBB69_1:
	ldaxrh	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB69_4
	stlxrh	w10, w9, [x2]
	cbnz	w10, .LBB69_1
	mov	w0, w8
	ret
.LBB69_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end69:
	.size	__aarch64_cas2_acq_rel, .Lfunc_end69-__aarch64_cas2_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_cas2_rel,"ax",@progbits
	.globl	__aarch64_cas2_rel
	.p2align	2
	.type	__aarch64_cas2_rel,@function
__aarch64_cas2_rel:
	.cfi_startproc
	mov	w9, w1
.LBB70_1:
	ldxrh	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB70_4
	stlxrh	w10, w9, [x2]
	cbnz	w10, .LBB70_1
	mov	w0, w8
	ret
.LBB70_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end70:
	.size	__aarch64_cas2_rel, .Lfunc_end70-__aarch64_cas2_rel
	.cfi_endproc

	.section	.text.__aarch64_cas2_relax,"ax",@progbits
	.globl	__aarch64_cas2_relax
	.p2align	2
	.type	__aarch64_cas2_relax,@function
__aarch64_cas2_relax:
	.cfi_startproc
	mov	w9, w1
.LBB71_1:
	ldxrh	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB71_4
	stxrh	w10, w9, [x2]
	cbnz	w10, .LBB71_1
	mov	w0, w8
	ret
.LBB71_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end71:
	.size	__aarch64_cas2_relax, .Lfunc_end71-__aarch64_cas2_relax
	.cfi_endproc

	.section	.text.__aarch64_cas4_acq,"ax",@progbits
	.globl	__aarch64_cas4_acq
	.p2align	2
	.type	__aarch64_cas4_acq,@function
__aarch64_cas4_acq:
	.cfi_startproc
	mov	w8, w0
.LBB72_1:
	ldaxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB72_4
	stxr	w9, w1, [x2]
	cbnz	w9, .LBB72_1
	ret
.LBB72_4:
	clrex
	ret
.Lfunc_end72:
	.size	__aarch64_cas4_acq, .Lfunc_end72-__aarch64_cas4_acq
	.cfi_endproc

	.section	.text.__aarch64_cas4_acq_rel,"ax",@progbits
	.globl	__aarch64_cas4_acq_rel
	.p2align	2
	.type	__aarch64_cas4_acq_rel,@function
__aarch64_cas4_acq_rel:
	.cfi_startproc
	mov	w8, w0
.LBB73_1:
	ldaxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB73_4
	stlxr	w9, w1, [x2]
	cbnz	w9, .LBB73_1
	ret
.LBB73_4:
	clrex
	ret
.Lfunc_end73:
	.size	__aarch64_cas4_acq_rel, .Lfunc_end73-__aarch64_cas4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_cas4_rel,"ax",@progbits
	.globl	__aarch64_cas4_rel
	.p2align	2
	.type	__aarch64_cas4_rel,@function
__aarch64_cas4_rel:
	.cfi_startproc
	mov	w8, w0
.LBB74_1:
	ldxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB74_4
	stlxr	w9, w1, [x2]
	cbnz	w9, .LBB74_1
	ret
.LBB74_4:
	clrex
	ret
.Lfunc_end74:
	.size	__aarch64_cas4_rel, .Lfunc_end74-__aarch64_cas4_rel
	.cfi_endproc

	.section	.text.__aarch64_cas4_relax,"ax",@progbits
	.globl	__aarch64_cas4_relax
	.p2align	2
	.type	__aarch64_cas4_relax,@function
__aarch64_cas4_relax:
	.cfi_startproc
	mov	w8, w0
.LBB75_1:
	ldxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB75_4
	stxr	w9, w1, [x2]
	cbnz	w9, .LBB75_1
	ret
.LBB75_4:
	clrex
	ret
.Lfunc_end75:
	.size	__aarch64_cas4_relax, .Lfunc_end75-__aarch64_cas4_relax
	.cfi_endproc

	.section	.text.__aarch64_cas8_acq,"ax",@progbits
	.globl	__aarch64_cas8_acq
	.p2align	2
	.type	__aarch64_cas8_acq,@function
__aarch64_cas8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB76_1:
	ldaxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB76_4
	stxr	w9, x1, [x2]
	cbnz	w9, .LBB76_1
	ret
.LBB76_4:
	clrex
	ret
.Lfunc_end76:
	.size	__aarch64_cas8_acq, .Lfunc_end76-__aarch64_cas8_acq
	.cfi_endproc

	.section	.text.__aarch64_cas8_acq_rel,"ax",@progbits
	.globl	__aarch64_cas8_acq_rel
	.p2align	2
	.type	__aarch64_cas8_acq_rel,@function
__aarch64_cas8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB77_1:
	ldaxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB77_4
	stlxr	w9, x1, [x2]
	cbnz	w9, .LBB77_1
	ret
.LBB77_4:
	clrex
	ret
.Lfunc_end77:
	.size	__aarch64_cas8_acq_rel, .Lfunc_end77-__aarch64_cas8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_cas8_rel,"ax",@progbits
	.globl	__aarch64_cas8_rel
	.p2align	2
	.type	__aarch64_cas8_rel,@function
__aarch64_cas8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB78_1:
	ldxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB78_4
	stlxr	w9, x1, [x2]
	cbnz	w9, .LBB78_1
	ret
.LBB78_4:
	clrex
	ret
.Lfunc_end78:
	.size	__aarch64_cas8_rel, .Lfunc_end78-__aarch64_cas8_rel
	.cfi_endproc

	.section	.text.__aarch64_cas8_relax,"ax",@progbits
	.globl	__aarch64_cas8_relax
	.p2align	2
	.type	__aarch64_cas8_relax,@function
__aarch64_cas8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB79_1:
	ldxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB79_4
	stxr	w9, x1, [x2]
	cbnz	w9, .LBB79_1
	ret
.LBB79_4:
	clrex
	ret
.Lfunc_end79:
	.size	__aarch64_cas8_relax, .Lfunc_end79-__aarch64_cas8_relax
	.cfi_endproc

	.section	.text.__aarch64_ldset1_acq,"ax",@progbits
	.globl	__aarch64_ldset1_acq
	.p2align	2
	.type	__aarch64_ldset1_acq,@function
__aarch64_ldset1_acq:
	.cfi_startproc
.LBB80_1:
	ldaxrb	w8, [x1]
	orr	w9, w8, w0
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB80_1
	mov	w0, w8
	ret
.Lfunc_end80:
	.size	__aarch64_ldset1_acq, .Lfunc_end80-__aarch64_ldset1_acq
	.cfi_endproc

	.section	.text.__aarch64_ldset1_acq_rel,"ax",@progbits
	.globl	__aarch64_ldset1_acq_rel
	.p2align	2
	.type	__aarch64_ldset1_acq_rel,@function
__aarch64_ldset1_acq_rel:
	.cfi_startproc
.LBB81_1:
	ldaxrb	w8, [x1]
	orr	w9, w8, w0
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB81_1
	mov	w0, w8
	ret
.Lfunc_end81:
	.size	__aarch64_ldset1_acq_rel, .Lfunc_end81-__aarch64_ldset1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset1_rel,"ax",@progbits
	.globl	__aarch64_ldset1_rel
	.p2align	2
	.type	__aarch64_ldset1_rel,@function
__aarch64_ldset1_rel:
	.cfi_startproc
.LBB82_1:
	ldxrb	w8, [x1]
	orr	w9, w8, w0
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB82_1
	mov	w0, w8
	ret
.Lfunc_end82:
	.size	__aarch64_ldset1_rel, .Lfunc_end82-__aarch64_ldset1_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset1_relax,"ax",@progbits
	.globl	__aarch64_ldset1_relax
	.p2align	2
	.type	__aarch64_ldset1_relax,@function
__aarch64_ldset1_relax:
	.cfi_startproc
.LBB83_1:
	ldxrb	w8, [x1]
	orr	w9, w8, w0
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB83_1
	mov	w0, w8
	ret
.Lfunc_end83:
	.size	__aarch64_ldset1_relax, .Lfunc_end83-__aarch64_ldset1_relax
	.cfi_endproc

	.section	.text.__aarch64_ldset2_acq,"ax",@progbits
	.globl	__aarch64_ldset2_acq
	.p2align	2
	.type	__aarch64_ldset2_acq,@function
__aarch64_ldset2_acq:
	.cfi_startproc
.LBB84_1:
	ldaxrh	w8, [x1]
	orr	w9, w8, w0
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB84_1
	mov	w0, w8
	ret
.Lfunc_end84:
	.size	__aarch64_ldset2_acq, .Lfunc_end84-__aarch64_ldset2_acq
	.cfi_endproc

	.section	.text.__aarch64_ldset2_acq_rel,"ax",@progbits
	.globl	__aarch64_ldset2_acq_rel
	.p2align	2
	.type	__aarch64_ldset2_acq_rel,@function
__aarch64_ldset2_acq_rel:
	.cfi_startproc
.LBB85_1:
	ldaxrh	w8, [x1]
	orr	w9, w8, w0
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB85_1
	mov	w0, w8
	ret
.Lfunc_end85:
	.size	__aarch64_ldset2_acq_rel, .Lfunc_end85-__aarch64_ldset2_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset2_rel,"ax",@progbits
	.globl	__aarch64_ldset2_rel
	.p2align	2
	.type	__aarch64_ldset2_rel,@function
__aarch64_ldset2_rel:
	.cfi_startproc
.LBB86_1:
	ldxrh	w8, [x1]
	orr	w9, w8, w0
	stlxrh	w10, w9, [x1]
	cbnz	w10, .LBB86_1
	mov	w0, w8
	ret
.Lfunc_end86:
	.size	__aarch64_ldset2_rel, .Lfunc_end86-__aarch64_ldset2_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset2_relax,"ax",@progbits
	.globl	__aarch64_ldset2_relax
	.p2align	2
	.type	__aarch64_ldset2_relax,@function
__aarch64_ldset2_relax:
	.cfi_startproc
.LBB87_1:
	ldxrh	w8, [x1]
	orr	w9, w8, w0
	stxrh	w10, w9, [x1]
	cbnz	w10, .LBB87_1
	mov	w0, w8
	ret
.Lfunc_end87:
	.size	__aarch64_ldset2_relax, .Lfunc_end87-__aarch64_ldset2_relax
	.cfi_endproc

	.section	.text.__aarch64_ldset4_acq,"ax",@progbits
	.globl	__aarch64_ldset4_acq
	.p2align	2
	.type	__aarch64_ldset4_acq,@function
__aarch64_ldset4_acq:
	.cfi_startproc
.LBB88_1:
	ldaxr	w8, [x1]
	orr	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB88_1
	mov	w0, w8
	ret
.Lfunc_end88:
	.size	__aarch64_ldset4_acq, .Lfunc_end88-__aarch64_ldset4_acq
	.cfi_endproc

	.section	.text.__aarch64_ldset4_acq_rel,"ax",@progbits
	.globl	__aarch64_ldset4_acq_rel
	.p2align	2
	.type	__aarch64_ldset4_acq_rel,@function
__aarch64_ldset4_acq_rel:
	.cfi_startproc
.LBB89_1:
	ldaxr	w8, [x1]
	orr	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB89_1
	mov	w0, w8
	ret
.Lfunc_end89:
	.size	__aarch64_ldset4_acq_rel, .Lfunc_end89-__aarch64_ldset4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset4_rel,"ax",@progbits
	.globl	__aarch64_ldset4_rel
	.p2align	2
	.type	__aarch64_ldset4_rel,@function
__aarch64_ldset4_rel:
	.cfi_startproc
.LBB90_1:
	ldxr	w8, [x1]
	orr	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB90_1
	mov	w0, w8
	ret
.Lfunc_end90:
	.size	__aarch64_ldset4_rel, .Lfunc_end90-__aarch64_ldset4_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset4_relax,"ax",@progbits
	.globl	__aarch64_ldset4_relax
	.p2align	2
	.type	__aarch64_ldset4_relax,@function
__aarch64_ldset4_relax:
	.cfi_startproc
.LBB91_1:
	ldxr	w8, [x1]
	orr	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB91_1
	mov	w0, w8
	ret
.Lfunc_end91:
	.size	__aarch64_ldset4_relax, .Lfunc_end91-__aarch64_ldset4_relax
	.cfi_endproc

	.section	.text.__aarch64_ldset8_acq,"ax",@progbits
	.globl	__aarch64_ldset8_acq
	.p2align	2
	.type	__aarch64_ldset8_acq,@function
__aarch64_ldset8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB92_1:
	ldaxr	x0, [x1]
	orr	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB92_1
	ret
.Lfunc_end92:
	.size	__aarch64_ldset8_acq, .Lfunc_end92-__aarch64_ldset8_acq
	.cfi_endproc

	.section	.text.__aarch64_ldset8_acq_rel,"ax",@progbits
	.globl	__aarch64_ldset8_acq_rel
	.p2align	2
	.type	__aarch64_ldset8_acq_rel,@function
__aarch64_ldset8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB93_1:
	ldaxr	x0, [x1]
	orr	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB93_1
	ret
.Lfunc_end93:
	.size	__aarch64_ldset8_acq_rel, .Lfunc_end93-__aarch64_ldset8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset8_rel,"ax",@progbits
	.globl	__aarch64_ldset8_rel
	.p2align	2
	.type	__aarch64_ldset8_rel,@function
__aarch64_ldset8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB94_1:
	ldxr	x0, [x1]
	orr	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB94_1
	ret
.Lfunc_end94:
	.size	__aarch64_ldset8_rel, .Lfunc_end94-__aarch64_ldset8_rel
	.cfi_endproc

	.section	.text.__aarch64_ldset8_relax,"ax",@progbits
	.globl	__aarch64_ldset8_relax
	.p2align	2
	.type	__aarch64_ldset8_relax,@function
__aarch64_ldset8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB95_1:
	ldxr	x0, [x1]
	orr	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB95_1
	ret
.Lfunc_end95:
	.size	__aarch64_ldset8_relax, .Lfunc_end95-__aarch64_ldset8_relax
	.cfi_endproc

	.section	".note.GNU-stack","",@progbits
