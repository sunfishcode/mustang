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

	.section	.text.__aarch64_ldadd4_acq,"ax",@progbits
	.globl	__aarch64_ldadd4_acq
	.p2align	2
	.type	__aarch64_ldadd4_acq,@function
__aarch64_ldadd4_acq:
	.cfi_startproc
.LBB4_1:
	ldaxr	w8, [x1]
	add	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB4_1
	mov	w0, w8
	ret
.Lfunc_end4:
	.size	__aarch64_ldadd4_acq, .Lfunc_end4-__aarch64_ldadd4_acq
	.cfi_endproc

	.section	.text.__aarch64_ldadd4_acq_rel,"ax",@progbits
	.globl	__aarch64_ldadd4_acq_rel
	.p2align	2
	.type	__aarch64_ldadd4_acq_rel,@function
__aarch64_ldadd4_acq_rel:
	.cfi_startproc
.LBB5_1:
	ldaxr	w8, [x1]
	add	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB5_1
	mov	w0, w8
	ret
.Lfunc_end5:
	.size	__aarch64_ldadd4_acq_rel, .Lfunc_end5-__aarch64_ldadd4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd4_rel,"ax",@progbits
	.globl	__aarch64_ldadd4_rel
	.p2align	2
	.type	__aarch64_ldadd4_rel,@function
__aarch64_ldadd4_rel:
	.cfi_startproc
.LBB6_1:
	ldxr	w8, [x1]
	add	w9, w8, w0
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB6_1
	mov	w0, w8
	ret
.Lfunc_end6:
	.size	__aarch64_ldadd4_rel, .Lfunc_end6-__aarch64_ldadd4_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd4_relax,"ax",@progbits
	.globl	__aarch64_ldadd4_relax
	.p2align	2
	.type	__aarch64_ldadd4_relax,@function
__aarch64_ldadd4_relax:
	.cfi_startproc
.LBB7_1:
	ldxr	w8, [x1]
	add	w9, w8, w0
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB7_1
	mov	w0, w8
	ret
.Lfunc_end7:
	.size	__aarch64_ldadd4_relax, .Lfunc_end7-__aarch64_ldadd4_relax
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_acq,"ax",@progbits
	.globl	__aarch64_ldadd8_acq
	.p2align	2
	.type	__aarch64_ldadd8_acq,@function
__aarch64_ldadd8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB8_1:
	ldaxr	x0, [x1]
	add	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB8_1
	ret
.Lfunc_end8:
	.size	__aarch64_ldadd8_acq, .Lfunc_end8-__aarch64_ldadd8_acq
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_acq_rel,"ax",@progbits
	.globl	__aarch64_ldadd8_acq_rel
	.p2align	2
	.type	__aarch64_ldadd8_acq_rel,@function
__aarch64_ldadd8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB9_1:
	ldaxr	x0, [x1]
	add	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB9_1
	ret
.Lfunc_end9:
	.size	__aarch64_ldadd8_acq_rel, .Lfunc_end9-__aarch64_ldadd8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_rel,"ax",@progbits
	.globl	__aarch64_ldadd8_rel
	.p2align	2
	.type	__aarch64_ldadd8_rel,@function
__aarch64_ldadd8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB10_1:
	ldxr	x0, [x1]
	add	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB10_1
	ret
.Lfunc_end10:
	.size	__aarch64_ldadd8_rel, .Lfunc_end10-__aarch64_ldadd8_rel
	.cfi_endproc

	.section	.text.__aarch64_ldadd8_relax,"ax",@progbits
	.globl	__aarch64_ldadd8_relax
	.p2align	2
	.type	__aarch64_ldadd8_relax,@function
__aarch64_ldadd8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB11_1:
	ldxr	x0, [x1]
	add	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB11_1
	ret
.Lfunc_end11:
	.size	__aarch64_ldadd8_relax, .Lfunc_end11-__aarch64_ldadd8_relax
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_acq,"ax",@progbits
	.globl	__aarch64_ldclr1_acq
	.p2align	2
	.type	__aarch64_ldclr1_acq,@function
__aarch64_ldclr1_acq:
	.cfi_startproc
	mvn	w8, w0
.LBB12_1:
	ldaxrb	w0, [x1]
	and	w9, w0, w8
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB12_1
	ret
.Lfunc_end12:
	.size	__aarch64_ldclr1_acq, .Lfunc_end12-__aarch64_ldclr1_acq
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_acq_rel,"ax",@progbits
	.globl	__aarch64_ldclr1_acq_rel
	.p2align	2
	.type	__aarch64_ldclr1_acq_rel,@function
__aarch64_ldclr1_acq_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB13_1:
	ldaxrb	w0, [x1]
	and	w9, w0, w8
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB13_1
	ret
.Lfunc_end13:
	.size	__aarch64_ldclr1_acq_rel, .Lfunc_end13-__aarch64_ldclr1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_rel,"ax",@progbits
	.globl	__aarch64_ldclr1_rel
	.p2align	2
	.type	__aarch64_ldclr1_rel,@function
__aarch64_ldclr1_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB14_1:
	ldxrb	w0, [x1]
	and	w9, w0, w8
	stlxrb	w10, w9, [x1]
	cbnz	w10, .LBB14_1
	ret
.Lfunc_end14:
	.size	__aarch64_ldclr1_rel, .Lfunc_end14-__aarch64_ldclr1_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr1_relax,"ax",@progbits
	.globl	__aarch64_ldclr1_relax
	.p2align	2
	.type	__aarch64_ldclr1_relax,@function
__aarch64_ldclr1_relax:
	.cfi_startproc
	mvn	w8, w0
.LBB15_1:
	ldxrb	w0, [x1]
	and	w9, w0, w8
	stxrb	w10, w9, [x1]
	cbnz	w10, .LBB15_1
	ret
.Lfunc_end15:
	.size	__aarch64_ldclr1_relax, .Lfunc_end15-__aarch64_ldclr1_relax
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_acq,"ax",@progbits
	.globl	__aarch64_ldclr4_acq
	.p2align	2
	.type	__aarch64_ldclr4_acq,@function
__aarch64_ldclr4_acq:
	.cfi_startproc
	mvn	w8, w0
.LBB16_1:
	ldaxr	w0, [x1]
	and	w9, w0, w8
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB16_1
	ret
.Lfunc_end16:
	.size	__aarch64_ldclr4_acq, .Lfunc_end16-__aarch64_ldclr4_acq
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_acq_rel,"ax",@progbits
	.globl	__aarch64_ldclr4_acq_rel
	.p2align	2
	.type	__aarch64_ldclr4_acq_rel,@function
__aarch64_ldclr4_acq_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB17_1:
	ldaxr	w0, [x1]
	and	w9, w0, w8
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB17_1
	ret
.Lfunc_end17:
	.size	__aarch64_ldclr4_acq_rel, .Lfunc_end17-__aarch64_ldclr4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_rel,"ax",@progbits
	.globl	__aarch64_ldclr4_rel
	.p2align	2
	.type	__aarch64_ldclr4_rel,@function
__aarch64_ldclr4_rel:
	.cfi_startproc
	mvn	w8, w0
.LBB18_1:
	ldxr	w0, [x1]
	and	w9, w0, w8
	stlxr	w10, w9, [x1]
	cbnz	w10, .LBB18_1
	ret
.Lfunc_end18:
	.size	__aarch64_ldclr4_rel, .Lfunc_end18-__aarch64_ldclr4_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr4_relax,"ax",@progbits
	.globl	__aarch64_ldclr4_relax
	.p2align	2
	.type	__aarch64_ldclr4_relax,@function
__aarch64_ldclr4_relax:
	.cfi_startproc
	mvn	w8, w0
.LBB19_1:
	ldxr	w0, [x1]
	and	w9, w0, w8
	stxr	w10, w9, [x1]
	cbnz	w10, .LBB19_1
	ret
.Lfunc_end19:
	.size	__aarch64_ldclr4_relax, .Lfunc_end19-__aarch64_ldclr4_relax
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_acq,"ax",@progbits
	.globl	__aarch64_ldclr8_acq
	.p2align	2
	.type	__aarch64_ldclr8_acq,@function
__aarch64_ldclr8_acq:
	.cfi_startproc
	mvn	x8, x0
.LBB20_1:
	ldaxr	x0, [x1]
	and	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB20_1
	ret
.Lfunc_end20:
	.size	__aarch64_ldclr8_acq, .Lfunc_end20-__aarch64_ldclr8_acq
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_acq_rel,"ax",@progbits
	.globl	__aarch64_ldclr8_acq_rel
	.p2align	2
	.type	__aarch64_ldclr8_acq_rel,@function
__aarch64_ldclr8_acq_rel:
	.cfi_startproc
	mvn	x8, x0
.LBB21_1:
	ldaxr	x0, [x1]
	and	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB21_1
	ret
.Lfunc_end21:
	.size	__aarch64_ldclr8_acq_rel, .Lfunc_end21-__aarch64_ldclr8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_rel,"ax",@progbits
	.globl	__aarch64_ldclr8_rel
	.p2align	2
	.type	__aarch64_ldclr8_rel,@function
__aarch64_ldclr8_rel:
	.cfi_startproc
	mvn	x8, x0
.LBB22_1:
	ldxr	x0, [x1]
	and	x9, x0, x8
	stlxr	w10, x9, [x1]
	cbnz	w10, .LBB22_1
	ret
.Lfunc_end22:
	.size	__aarch64_ldclr8_rel, .Lfunc_end22-__aarch64_ldclr8_rel
	.cfi_endproc

	.section	.text.__aarch64_ldclr8_relax,"ax",@progbits
	.globl	__aarch64_ldclr8_relax
	.p2align	2
	.type	__aarch64_ldclr8_relax,@function
__aarch64_ldclr8_relax:
	.cfi_startproc
	mvn	x8, x0
.LBB23_1:
	ldxr	x0, [x1]
	and	x9, x0, x8
	stxr	w10, x9, [x1]
	cbnz	w10, .LBB23_1
	ret
.Lfunc_end23:
	.size	__aarch64_ldclr8_relax, .Lfunc_end23-__aarch64_ldclr8_relax
	.cfi_endproc

	.section	.text.__aarch64_swp1_acq,"ax",@progbits
	.globl	__aarch64_swp1_acq
	.p2align	2
	.type	__aarch64_swp1_acq,@function
__aarch64_swp1_acq:
	.cfi_startproc
	mov	w8, w0
.LBB24_1:
	ldaxrb	w0, [x1]
	stxrb	w9, w8, [x1]
	cbnz	w9, .LBB24_1
	ret
.Lfunc_end24:
	.size	__aarch64_swp1_acq, .Lfunc_end24-__aarch64_swp1_acq
	.cfi_endproc

	.section	.text.__aarch64_swp1_acq_rel,"ax",@progbits
	.globl	__aarch64_swp1_acq_rel
	.p2align	2
	.type	__aarch64_swp1_acq_rel,@function
__aarch64_swp1_acq_rel:
	.cfi_startproc
	mov	w8, w0
.LBB25_1:
	ldaxrb	w0, [x1]
	stlxrb	w9, w8, [x1]
	cbnz	w9, .LBB25_1
	ret
.Lfunc_end25:
	.size	__aarch64_swp1_acq_rel, .Lfunc_end25-__aarch64_swp1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_swp1_rel,"ax",@progbits
	.globl	__aarch64_swp1_rel
	.p2align	2
	.type	__aarch64_swp1_rel,@function
__aarch64_swp1_rel:
	.cfi_startproc
	mov	w8, w0
.LBB26_1:
	ldxrb	w0, [x1]
	stlxrb	w9, w8, [x1]
	cbnz	w9, .LBB26_1
	ret
.Lfunc_end26:
	.size	__aarch64_swp1_rel, .Lfunc_end26-__aarch64_swp1_rel
	.cfi_endproc

	.section	.text.__aarch64_swp1_relax,"ax",@progbits
	.globl	__aarch64_swp1_relax
	.p2align	2
	.type	__aarch64_swp1_relax,@function
__aarch64_swp1_relax:
	.cfi_startproc
	mov	w8, w0
.LBB27_1:
	ldxrb	w0, [x1]
	stxrb	w9, w8, [x1]
	cbnz	w9, .LBB27_1
	ret
.Lfunc_end27:
	.size	__aarch64_swp1_relax, .Lfunc_end27-__aarch64_swp1_relax
	.cfi_endproc

	.section	.text.__aarch64_swp4_acq,"ax",@progbits
	.globl	__aarch64_swp4_acq
	.p2align	2
	.type	__aarch64_swp4_acq,@function
__aarch64_swp4_acq:
	.cfi_startproc
	mov	w8, w0
.LBB28_1:
	ldaxr	w0, [x1]
	stxr	w9, w8, [x1]
	cbnz	w9, .LBB28_1
	ret
.Lfunc_end28:
	.size	__aarch64_swp4_acq, .Lfunc_end28-__aarch64_swp4_acq
	.cfi_endproc

	.section	.text.__aarch64_swp4_acq_rel,"ax",@progbits
	.globl	__aarch64_swp4_acq_rel
	.p2align	2
	.type	__aarch64_swp4_acq_rel,@function
__aarch64_swp4_acq_rel:
	.cfi_startproc
	mov	w8, w0
.LBB29_1:
	ldaxr	w0, [x1]
	stlxr	w9, w8, [x1]
	cbnz	w9, .LBB29_1
	ret
.Lfunc_end29:
	.size	__aarch64_swp4_acq_rel, .Lfunc_end29-__aarch64_swp4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_swp4_rel,"ax",@progbits
	.globl	__aarch64_swp4_rel
	.p2align	2
	.type	__aarch64_swp4_rel,@function
__aarch64_swp4_rel:
	.cfi_startproc
	mov	w8, w0
.LBB30_1:
	ldxr	w0, [x1]
	stlxr	w9, w8, [x1]
	cbnz	w9, .LBB30_1
	ret
.Lfunc_end30:
	.size	__aarch64_swp4_rel, .Lfunc_end30-__aarch64_swp4_rel
	.cfi_endproc

	.section	.text.__aarch64_swp4_relax,"ax",@progbits
	.globl	__aarch64_swp4_relax
	.p2align	2
	.type	__aarch64_swp4_relax,@function
__aarch64_swp4_relax:
	.cfi_startproc
	mov	w8, w0
.LBB31_1:
	ldxr	w0, [x1]
	stxr	w9, w8, [x1]
	cbnz	w9, .LBB31_1
	ret
.Lfunc_end31:
	.size	__aarch64_swp4_relax, .Lfunc_end31-__aarch64_swp4_relax
	.cfi_endproc

	.section	.text.__aarch64_swp8_acq,"ax",@progbits
	.globl	__aarch64_swp8_acq
	.p2align	2
	.type	__aarch64_swp8_acq,@function
__aarch64_swp8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB32_1:
	ldaxr	x0, [x1]
	stxr	w9, x8, [x1]
	cbnz	w9, .LBB32_1
	ret
.Lfunc_end32:
	.size	__aarch64_swp8_acq, .Lfunc_end32-__aarch64_swp8_acq
	.cfi_endproc

	.section	.text.__aarch64_swp8_acq_rel,"ax",@progbits
	.globl	__aarch64_swp8_acq_rel
	.p2align	2
	.type	__aarch64_swp8_acq_rel,@function
__aarch64_swp8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB33_1:
	ldaxr	x0, [x1]
	stlxr	w9, x8, [x1]
	cbnz	w9, .LBB33_1
	ret
.Lfunc_end33:
	.size	__aarch64_swp8_acq_rel, .Lfunc_end33-__aarch64_swp8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_swp8_rel,"ax",@progbits
	.globl	__aarch64_swp8_rel
	.p2align	2
	.type	__aarch64_swp8_rel,@function
__aarch64_swp8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB34_1:
	ldxr	x0, [x1]
	stlxr	w9, x8, [x1]
	cbnz	w9, .LBB34_1
	ret
.Lfunc_end34:
	.size	__aarch64_swp8_rel, .Lfunc_end34-__aarch64_swp8_rel
	.cfi_endproc

	.section	.text.__aarch64_swp8_relax,"ax",@progbits
	.globl	__aarch64_swp8_relax
	.p2align	2
	.type	__aarch64_swp8_relax,@function
__aarch64_swp8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB35_1:
	ldxr	x0, [x1]
	stxr	w9, x8, [x1]
	cbnz	w9, .LBB35_1
	ret
.Lfunc_end35:
	.size	__aarch64_swp8_relax, .Lfunc_end35-__aarch64_swp8_relax
	.cfi_endproc

	.section	.text.__aarch64_cas1_acq,"ax",@progbits
	.globl	__aarch64_cas1_acq
	.p2align	2
	.type	__aarch64_cas1_acq,@function
__aarch64_cas1_acq:
	.cfi_startproc
	mov	w9, w1
.LBB36_1:
	ldaxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB36_4
	stxrb	w10, w9, [x2]
	cbnz	w10, .LBB36_1
	mov	w0, w8
	ret
.LBB36_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end36:
	.size	__aarch64_cas1_acq, .Lfunc_end36-__aarch64_cas1_acq
	.cfi_endproc

	.section	.text.__aarch64_cas1_acq_rel,"ax",@progbits
	.globl	__aarch64_cas1_acq_rel
	.p2align	2
	.type	__aarch64_cas1_acq_rel,@function
__aarch64_cas1_acq_rel:
	.cfi_startproc
	mov	w9, w1
.LBB37_1:
	ldaxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB37_4
	stlxrb	w10, w9, [x2]
	cbnz	w10, .LBB37_1
	mov	w0, w8
	ret
.LBB37_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end37:
	.size	__aarch64_cas1_acq_rel, .Lfunc_end37-__aarch64_cas1_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_cas1_rel,"ax",@progbits
	.globl	__aarch64_cas1_rel
	.p2align	2
	.type	__aarch64_cas1_rel,@function
__aarch64_cas1_rel:
	.cfi_startproc
	mov	w9, w1
.LBB38_1:
	ldxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB38_4
	stlxrb	w10, w9, [x2]
	cbnz	w10, .LBB38_1
	mov	w0, w8
	ret
.LBB38_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end38:
	.size	__aarch64_cas1_rel, .Lfunc_end38-__aarch64_cas1_rel
	.cfi_endproc

	.section	.text.__aarch64_cas1_relax,"ax",@progbits
	.globl	__aarch64_cas1_relax
	.p2align	2
	.type	__aarch64_cas1_relax,@function
__aarch64_cas1_relax:
	.cfi_startproc
	mov	w9, w1
.LBB39_1:
	ldxrb	w8, [x2]
	cmp	w8, w0
	b.ne	.LBB39_4
	stxrb	w10, w9, [x2]
	cbnz	w10, .LBB39_1
	mov	w0, w8
	ret
.LBB39_4:
	clrex
	mov	w0, w8
	ret
.Lfunc_end39:
	.size	__aarch64_cas1_relax, .Lfunc_end39-__aarch64_cas1_relax
	.cfi_endproc

	.section	.text.__aarch64_cas4_acq,"ax",@progbits
	.globl	__aarch64_cas4_acq
	.p2align	2
	.type	__aarch64_cas4_acq,@function
__aarch64_cas4_acq:
	.cfi_startproc
	mov	w8, w0
.LBB40_1:
	ldaxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB40_4
	stxr	w9, w1, [x2]
	cbnz	w9, .LBB40_1
	ret
.LBB40_4:
	clrex
	ret
.Lfunc_end40:
	.size	__aarch64_cas4_acq, .Lfunc_end40-__aarch64_cas4_acq
	.cfi_endproc

	.section	.text.__aarch64_cas4_acq_rel,"ax",@progbits
	.globl	__aarch64_cas4_acq_rel
	.p2align	2
	.type	__aarch64_cas4_acq_rel,@function
__aarch64_cas4_acq_rel:
	.cfi_startproc
	mov	w8, w0
.LBB41_1:
	ldaxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB41_4
	stlxr	w9, w1, [x2]
	cbnz	w9, .LBB41_1
	ret
.LBB41_4:
	clrex
	ret
.Lfunc_end41:
	.size	__aarch64_cas4_acq_rel, .Lfunc_end41-__aarch64_cas4_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_cas4_rel,"ax",@progbits
	.globl	__aarch64_cas4_rel
	.p2align	2
	.type	__aarch64_cas4_rel,@function
__aarch64_cas4_rel:
	.cfi_startproc
	mov	w8, w0
.LBB42_1:
	ldxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB42_4
	stlxr	w9, w1, [x2]
	cbnz	w9, .LBB42_1
	ret
.LBB42_4:
	clrex
	ret
.Lfunc_end42:
	.size	__aarch64_cas4_rel, .Lfunc_end42-__aarch64_cas4_rel
	.cfi_endproc

	.section	.text.__aarch64_cas4_relax,"ax",@progbits
	.globl	__aarch64_cas4_relax
	.p2align	2
	.type	__aarch64_cas4_relax,@function
__aarch64_cas4_relax:
	.cfi_startproc
	mov	w8, w0
.LBB43_1:
	ldxr	w0, [x2]
	cmp	w0, w8
	b.ne	.LBB43_4
	stxr	w9, w1, [x2]
	cbnz	w9, .LBB43_1
	ret
.LBB43_4:
	clrex
	ret
.Lfunc_end43:
	.size	__aarch64_cas4_relax, .Lfunc_end43-__aarch64_cas4_relax
	.cfi_endproc

	.section	.text.__aarch64_cas8_acq,"ax",@progbits
	.globl	__aarch64_cas8_acq
	.p2align	2
	.type	__aarch64_cas8_acq,@function
__aarch64_cas8_acq:
	.cfi_startproc
	mov	x8, x0
.LBB44_1:
	ldaxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB44_4
	stxr	w9, x1, [x2]
	cbnz	w9, .LBB44_1
	ret
.LBB44_4:
	clrex
	ret
.Lfunc_end44:
	.size	__aarch64_cas8_acq, .Lfunc_end44-__aarch64_cas8_acq
	.cfi_endproc

	.section	.text.__aarch64_cas8_acq_rel,"ax",@progbits
	.globl	__aarch64_cas8_acq_rel
	.p2align	2
	.type	__aarch64_cas8_acq_rel,@function
__aarch64_cas8_acq_rel:
	.cfi_startproc
	mov	x8, x0
.LBB45_1:
	ldaxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB45_4
	stlxr	w9, x1, [x2]
	cbnz	w9, .LBB45_1
	ret
.LBB45_4:
	clrex
	ret
.Lfunc_end45:
	.size	__aarch64_cas8_acq_rel, .Lfunc_end45-__aarch64_cas8_acq_rel
	.cfi_endproc

	.section	.text.__aarch64_cas8_rel,"ax",@progbits
	.globl	__aarch64_cas8_rel
	.p2align	2
	.type	__aarch64_cas8_rel,@function
__aarch64_cas8_rel:
	.cfi_startproc
	mov	x8, x0
.LBB46_1:
	ldxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB46_4
	stlxr	w9, x1, [x2]
	cbnz	w9, .LBB46_1
	ret
.LBB46_4:
	clrex
	ret
.Lfunc_end46:
	.size	__aarch64_cas8_rel, .Lfunc_end46-__aarch64_cas8_rel
	.cfi_endproc

	.section	.text.__aarch64_cas8_relax,"ax",@progbits
	.globl	__aarch64_cas8_relax
	.p2align	2
	.type	__aarch64_cas8_relax,@function
__aarch64_cas8_relax:
	.cfi_startproc
	mov	x8, x0
.LBB47_1:
	ldxr	x0, [x2]
	cmp	x0, x8
	b.ne	.LBB47_4
	stxr	w9, x1, [x2]
	cbnz	w9, .LBB47_1
	ret
.LBB47_4:
	clrex
	ret
.Lfunc_end47:
	.size	__aarch64_cas8_relax, .Lfunc_end47-__aarch64_cas8_relax
	.cfi_endproc

	.section	".note.GNU-stack","",@progbits
