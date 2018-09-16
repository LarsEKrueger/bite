/*
    BiTE - Bash-integrated Terminal
    Copyright (C) 2018  Lars Krüger

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.

    These tables have been adapted from:
    $XTermId: VTPrsTbl.c,v 1.81 2015/02/16 01:51:51 tom Exp $

    with the following license:

 * Copyright 1999-2014,2015 by Thomas E. Dickey
 *
 *                         All Rights Reserved
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE ABOVE LISTED COPYRIGHT HOLDER(S) BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 * Except as contained in this notice, the name(s) of the above copyright
 * holders shall not be used in advertising or otherwise to promote the
 * sale, use or other dealings in this Software without prior written
 * authorization.
 *
 *
 * Copyright 1987 by Digital Equipment Corporation, Maynard, Massachusetts.
 *
 *                         All Rights Reserved
 *
 * Permission to use, copy, modify, and distribute this software and its
 * documentation for any purpose and without fee is hereby granted,
 * provided that the above copyright notice appear in all copies and that
 * both that copyright notice and this permission notice appear in
 * supporting documentation, and that the name of Digital Equipment
 * Corporation not be used in advertising or publicity pertaining to
 * distribution of the software without specific, written prior permission.
 *
 *
 * DIGITAL DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE, INCLUDING
 * ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS, IN NO EVENT SHALL
 * DIGITAL BE LIABLE FOR ANY SPECIAL, INDIRECT OR CONSEQUENTIAL DAMAGES OR
 * ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
 * WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
 * ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS
 * SOFTWARE.
*/

use super::types::{Case, CaseTable};

/*
 * Stupid Apollo C preprocessor can't handle long lines.  So... To keep
 * it happy, we put each onto a separate line....  Sigh...
 */

pub static ansi_table: CaseTable = [
    /*	NUL		SOH		STX		ETX	*/
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    /*	EOT		ENQ		ACK		BEL	*/
    Case::IGNORE,
    Case::ENQ,
    Case::IGNORE,
    Case::BELL,
    /*	BS		HT		NL		VT	*/
    Case::BS,
    Case::TAB,
    Case::VMOT,
    Case::VMOT,
    /*	FF		CR		SO		SI	*/
    Case::VMOT,
    Case::CR,
    Case::SO,
    Case::SI,
    /*	DLE		DC1		DC2		DC3	*/
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    /*	DC4		NAK		SYN		ETB	*/
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    /*	CAN		EM		SUB		ESC	*/
    Case::GROUND_STATE,
    Case::IGNORE,
    Case::GROUND_STATE,
    Case::ESC,
    /*	FS		GS		RS		US	*/
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    Case::IGNORE,
    /*	SP		!		"		#	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	$		%		&		'	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	(		)		*		+	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	,		-		.		/	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	0		1		2		3	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	4		5		6		7	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	8		9		:		;	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	<		=		>		?	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	@		A		B		C	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	D		E		F		G	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	H		I		J		K	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	L		M		N		O	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	P		Q		R		S	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	T		U		V		W	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	X		Y		Z		[	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	\		]		^		_	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	`		a		b		c	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	d		e		f		g	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	h		i		j		k	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	l		m		n		o	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	p		q		r		s	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	t		u		v		w	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	x		y		z		{	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    /*	|		}		~		DEL	*/
    Case::PRINT,
    Case::PRINT,
    Case::PRINT,
    Case::IGNORE,
];

pub static csi_table:CaseTable =		/* CSI */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_SPACE_STATE,
Case::CSI_EX_STATE,
Case::CSI_QUOTE_STATE,
Case::CSI_HASH_STATE,
/*	$		%		&		'	*/
Case::CSI_DOLLAR_STATE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_TICK_STATE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	4		5		6		7	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	8		9		:		;	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_COLON,
Case::ESC_SEMI,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::DEC3_STATE,
Case::DEC2_STATE,
Case::DEC_STATE,
/*	@		A		B		C	*/
Case::ICH,
Case::CUU,
Case::CUD,
Case::CUF,
/*	D		E		F		G	*/
Case::CUB,
Case::CNL,
Case::CPL,
Case::HPA,
/*	H		I		J		K	*/
Case::CUP,
Case::CHT,
Case::ED,
Case::EL,
/*	L		M		N		O	*/
Case::IL,
Case::DL,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::DCH,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::SU,
/*	T		U		V		W	*/
Case::TRACK_MOUSE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::ECH,
Case::GROUND_STATE,
Case::CBT,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::SD,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::HPA,
Case::HPR,
Case::REP,
Case::DA1,
/*	d		e		f		g	*/
Case::VPA,
Case::VPR,
Case::CUP,
Case::TBC,
/*	h		i		j		k	*/
Case::SET,
Case::MC,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::RST,
Case::SGR,
Case::CPR,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::DECLL,
Case::DECSTBM,
Case::ANSI_SC,
/*	t		u		v		w	*/
Case::XTERM_WINOPS,
Case::ANSI_RC,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::DECREQTPARM,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static csi2_table:CaseTable =		/* CSI */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_SPACE_STATE,
Case::CSI_EX_STATE,
Case::CSI_QUOTE_STATE,
Case::CSI_HASH_STATE,
/*	$		%		&		'	*/
Case::CSI_DOLLAR_STATE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_TICK_STATE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_STAR_STATE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	4		5		6		7	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	8		9		:		;	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_COLON,
Case::ESC_SEMI,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::ICH,
Case::CUU,
Case::CUD,
Case::CUF,
/*	D		E		F		G	*/
Case::CUB,
Case::CNL,
Case::CPL,
Case::HPA,
/*	H		I		J		K	*/
Case::CUP,
Case::CHT,
Case::ED,
Case::EL,
/*	L		M		N		O	*/
Case::IL,
Case::DL,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::DCH,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::SU,
/*	T		U		V		W	*/
Case::TRACK_MOUSE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::ECH,
Case::GROUND_STATE,
Case::CBT,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::SD,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::HPA,
Case::HPR,
Case::REP,
Case::DA1,
/*	d		e		f		g	*/
Case::VPA,
Case::VPR,
Case::CUP,
Case::TBC,
/*	h		i		j		k	*/
Case::SET,
Case::MC,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::RST,
Case::SGR,
Case::CPR,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::DECLL,
Case::DECSTBM,
Case::ANSI_SC,
/*	t		u		v		w	*/
Case::XTERM_WINOPS,
Case::ANSI_RC,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::DECREQTPARM,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static csi_ex_table:CaseTable =		/* CSI ! */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::DECSTR,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static csi_quo_table:CaseTable =		/* CSI ... " */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::DECSCL,
Case::DECSCA,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static csi_sp_table:CaseTable =		/* CSI ... SP */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::SL,
Case::SR,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::DECSCUSR,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::DECSWBV,
Case::DECSMBV,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static csi_tick_table:CaseTable =	/* CSI ... ' */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::DECEFR,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::DECELR,
Case::DECSLE,
/*	|		}		~		DEL	*/
Case::DECRQLP,
Case::DECIC,
Case::DECDC,
Case::IGNORE,
];

pub static csi_hash_table:CaseTable =	/* CSI ... # */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::XTERM_PUSH_SGR,
/*	|		}		~		DEL	*/
Case::XTERM_REPORT_SGR,
Case::XTERM_POP_SGR,
Case::GROUND_STATE,
Case::IGNORE,
];


pub static csi_dollar_table:CaseTable =	/* CSI ... $ */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::RQM,
Case::GROUND_STATE,
Case::DECCARA,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::DECRARA,
Case::GROUND_STATE,
Case::DECCRA,
Case::DECRQPSR,
/*	x		y		z		{	*/
Case::DECFRA,
Case::GROUND_STATE,
Case::DECERA,
Case::DECSERA,
/*	|		}		~		DEL	*/
Case::DECSCPP,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static csi_star_table:CaseTable =	/* CSI ... * */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::DECSACE,
Case::DECRQCRA,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::DECSNLS,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];


#[allow(dead_code)]
pub static dec_table:CaseTable =		/* CSI ? */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_DEC_DOLLAR_STATE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	4		5		6		7	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	8		9		:		;	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_COLON,
Case::ESC_SEMI,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::DECSED,
Case::DECSEL,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GRAPHICS_ATTRIBUTES,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::DECSET,
Case::DEC_MC,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::DECRST,
Case::GROUND_STATE,
Case::DSR,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::XTERM_RESTORE,
Case::XTERM_SAVE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static csi_dec_dollar_table:CaseTable =	/* CSI ?... $ */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	4		5		6		7	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	8		9		:		;	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::DECRQM,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];


#[allow(dead_code)]
pub static dec2_table:CaseTable =		/* CSI > */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	4		5		6		7	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	8		9		:		;	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_COLON,
Case::ESC_SEMI,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::RM_TITLE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::DA2,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::SET_MOD_FKEYS,
Case::SET_MOD_FKEYS0,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::HIDE_POINTER,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::SM_TITLE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static dec3_table:CaseTable =		/* CSI = */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	$		%		&		'	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	(		)		*		+	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	,		-		.		/	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	0		1		2		3	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	4		5		6		7	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_DIGIT,
/*	8		9		:		;	*/
Case::ESC_DIGIT,
Case::ESC_DIGIT,
Case::ESC_COLON,
Case::ESC_SEMI,
/*	<		=		>		?	*/
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
Case::CSI_IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::DECRPTUI,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static cigtable:CaseTable =		/* CASE_CSI_IGNORE */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	$		%		&		'	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	(		)		*		+	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	,		-		.		/	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	0		1		2		3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	4		5		6		7	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	8		9		:		;	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	<		=		>		?	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

#[allow(dead_code)]
pub static eigtable:CaseTable =		/* CASE_ESC_IGNORE */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	$		%		&		'	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	(		)		*		+	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	,		-		.		/	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	0		1		2		3	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	4		5		6		7	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	8		9		:		;	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static esc_table:CaseTable =		/* ESC */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::ESC_SP_STATE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::SCR_STATE,
/*	$		%		&		'	*/
Case::ESC_IGNORE,
Case::ESC_PERCENT,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	(		)		*		+	*/
Case::SCS0_STATE,
Case::SCS1_STATE,
Case::SCS2_STATE,
Case::SCS3_STATE,
/*	,		-		.		/	*/
Case::ESC_IGNORE,
Case::SCS1A_STATE,
Case::SCS2A_STATE,
Case::SCS3A_STATE,
/*	0		1		2		3	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	4		5		6		7	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::DECBI,
Case::DECSC,
/*	8		9		:		;	*/
Case::DECRC,
Case::DECFI,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GROUND_STATE,
Case::DECKPAM,
Case::DECKPNM,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::IND,
Case::NEL,
Case::HP_BUGGY_LL,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::HTS,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::RI,
Case::SS2,
Case::SS3,
/*	P		Q		R		S	*/
Case::DCS,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::XTERM_TITLE,
Case::GROUND_STATE,
Case::SPA,
Case::EPA,
/*	X		Y		Z		[	*/
Case::SOS,
Case::GROUND_STATE,
Case::DECID,
Case::CSI_STATE,
/*	\		]		^		_	*/
Case::ST,
Case::OSC,
Case::PM,
Case::APC,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::RIS,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::HP_MEM_LOCK,
Case::HP_MEM_UNLOCK,
Case::LS2,
Case::LS3,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::LS3R,
Case::LS2R,
Case::LS1R,
Case::IGNORE,
];

pub static esc_sp_table:CaseTable =		/* ESC SP */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	$		%		&		'	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	(		)		*		+	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	,		-		.		/	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	0		1		2		3	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	4		5		6		7	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	8		9		:		;	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::S7C1T,
Case::S8C1T,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::ANSI_LEVEL_1,
Case::ANSI_LEVEL_2,
Case::ANSI_LEVEL_3,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static scrtable:CaseTable =		/* ESC # */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	$		%		&		'	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	(		)		*		+	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	,		-		.		/	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	0		1		2		3	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::DECDHL,
/*	4		5		6		7	*/
Case::DECDHL,
Case::DECSWL,
Case::DECDWL,
Case::GROUND_STATE,
/*	8		9		:		;	*/
Case::DECALN,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static scstable:CaseTable =		/* ESC ( etc. */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	$		%		&		'	*/
Case::ESC_IGNORE,
Case::SCS_PERCENT,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	(		)		*		+	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	,		-		.		/	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	0		1		2		3	*/
Case::GSETS,
Case::GSETS,
Case::GSETS,
Case::GROUND_STATE,
/*	4		5		6		7	*/
Case::GSETS,
Case::GSETS,
Case::GSETS,
Case::GSETS,
/*	8		9		:		;	*/
Case::GROUND_STATE,
Case::GSETS,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GSETS,
Case::GSETS,
Case::GSETS,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GSETS,
Case::GSETS,
Case::GSETS,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GSETS,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GSETS,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GSETS,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GSETS,
Case::GSETS,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GSETS,
Case::GSETS,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GSETS,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GSETS,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static scs96table:CaseTable =		/* ESC - etc. */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	$		%		&		'	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	(		)		*		+	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	,		-		.		/	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	0		1		2		3	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	4		5		6		7	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	8		9		:		;	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GSETS,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

/*
 * This table is treated specially.  The CASE_IGNORE entries correspond to the
 * characters that can be accumulated for the string function (e.g., OSC).
 */
pub static sos_table:CaseTable =		/* OSC, DCS, etc. */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	FF		CR		SO		SI	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	$		%		&		'	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	(		)		*		+	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	,		-		.		/	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	0		1		2		3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	4		5		6		7	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	8		9		:		;	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	<		=		>		?	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	@		A		B		C	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	D		E		F		G	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	H		I		J		K	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	L		M		N		O	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	P		Q		R		S	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	T		U		V		W	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	X		Y		Z		[	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	\		]		^		_	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	`		a		b		c	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	d		e		f		g	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	h		i		j		k	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	l		m		n		o	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	p		q		r		s	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	t		u		v		w	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	x		y		z		{	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	|		}		~		DEL	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
];

pub static esc_pct_table:CaseTable =		/* ESC % */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	$		%		&		'	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	(		)		*		+	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	,		-		.		/	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	0		1		2		3	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	4		5		6		7	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	8		9		:		;	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::UTF8,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::UTF8,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];

pub static scs_pct_table:CaseTable =		/* SCS % */
[
/*	NUL		SOH		STX		ETX	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	EOT		ENQ		ACK		BEL	*/
Case::IGNORE,
Case::ENQ,
Case::IGNORE,
Case::BELL,
/*	BS		HT		NL		VT	*/
Case::BS,
Case::TAB,
Case::VMOT,
Case::VMOT,
/*	FF		CR		SO		SI	*/
Case::VMOT,
Case::CR,
Case::SO,
Case::SI,
/*	DLE		DC1		DC2		DC3	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	DC4		NAK		SYN		ETB	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	CAN		EM		SUB		ESC	*/
Case::GROUND_STATE,
Case::IGNORE,
Case::GROUND_STATE,
Case::ESC,
/*	FS		GS		RS		US	*/
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
Case::IGNORE,
/*	SP		!		"		#	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	$		%		&		'	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	(		)		*		+	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	,		-		.		/	*/
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
Case::ESC_IGNORE,
/*	0		1		2		3	*/
Case::GSETS_PERCENT,
Case::GROUND_STATE,
Case::GSETS_PERCENT,
Case::GSETS_PERCENT,
/*	4		5		6		7	*/
Case::GROUND_STATE,
Case::GSETS_PERCENT,
Case::GSETS_PERCENT,
Case::GROUND_STATE,
/*	8		9		:		;	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	<		=		>		?	*/
Case::GROUND_STATE,
Case::GSETS_PERCENT,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	@		A		B		C	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	D		E		F		G	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	H		I		J		K	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	L		M		N		O	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	P		Q		R		S	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	T		U		V		W	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	X		Y		Z		[	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	\		]		^		_	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	`		a		b		c	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	d		e		f		g	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	h		i		j		k	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	l		m		n		o	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	p		q		r		s	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	t		u		v		w	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	x		y		z		{	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
/*	|		}		~		DEL	*/
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::GROUND_STATE,
Case::IGNORE,
];
