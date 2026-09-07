
xxdd.exe:	file format coff-i386

Disassembly of section .text:

00401000 <.text>:
  401000: e8 cc 00 00 00               	calll	0x4010d1 <.text+0xd1>
  401005: 80 3d c8 21 40 00 01         	cmpb	$0x1, 0x4021c8
  40100c: 0f 82 b8 00 00 00            	jb	0x4010ca <.text+0xca>
  401012: 6a f5                        	pushl	$-0xb
  401014: e8 87 03 00 00               	calll	0x4013a0 <.text+0x3a0>
  401019: a3 68 20 40 00               	movl	%eax, 0x402068
  40101e: 6a 00                        	pushl	$0x0
  401020: 68 74 20 40 00               	pushl	$0x402074               # imm = 0x402074
  401025: 68 c9 21 40 00               	pushl	$0x4021c9               # imm = 0x4021C9
  40102a: e8 77 03 00 00               	calll	0x4013a6 <.text+0x3a6>
  40102f: a3 fc 20 40 00               	movl	%eax, 0x4020fc
  401034: 68 00 10 00 00               	pushl	$0x1000                 # imm = 0x1000
  401039: 68 04 21 40 00               	pushl	$0x402104               # imm = 0x402104
  40103e: 68 d2 21 40 00               	pushl	$0x4021d2               # imm = 0x4021D2
  401043: e8 5e 03 00 00               	calll	0x4013a6 <.text+0x3a6>
  401048: a3 8c 21 40 00               	movl	%eax, 0x40218c
  40104d: 6a 00                        	pushl	$0x0
  40104f: 68 00 21 40 00               	pushl	$0x402100               # imm = 0x402100
  401054: 68 00 00 0a 00               	pushl	$0xa0000                # imm = 0xA0000
  401059: 68 50 23 40 00               	pushl	$0x402350               # imm = 0x402350
  40105e: ff 35 fc 20 40 00            	pushl	0x4020fc
  401064: e8 43 03 00 00               	calll	0x4013ac <.text+0x3ac>
  401069: be 50 23 40 00               	movl	$0x402350, %esi         # imm = 0x402350
  40106e: bf 50 23 4a 00               	movl	$0x4a2350, %edi         # imm = 0x4A2350
  401073: b9 00 00 01 00               	movl	$0x10000, %ecx          # imm = 0x10000
  401078: 56                           	pushl	%esi
  401079: 51                           	pushl	%ecx
  40107a: 57                           	pushl	%edi
  40107b: e8 80 01 00 00               	calll	0x401200 <.text+0x200>
  401080: 5f                           	popl	%edi
  401081: e8 11 01 00 00               	calll	0x401197 <.text+0x197>
  401086: 59                           	popl	%ecx
  401087: 5e                           	popl	%esi
  401088: ff 05 54 20 40 00            	incl	0x402054
  40108e: a5                           	movsl	(%esi), %es:(%edi)
  40108f: a5                           	movsl	(%esi), %es:(%edi)
  401090: 66 a5                        	movsw	(%esi), %es:(%edi)
  401092: e2 e4                        	loop	0x401078 <.text+0x78>
  401094: 6a 00                        	pushl	$0x0
  401096: 68 90 21 40 00               	pushl	$0x402190               # imm = 0x402190
  40109b: 68 00 00 17 00               	pushl	$0x170000               # imm = 0x170000
  4010a0: 68 50 23 4a 00               	pushl	$0x4a2350               # imm = 0x4A2350
  4010a5: ff 35 8c 21 40 00            	pushl	0x40218c
  4010ab: e8 02 03 00 00               	calll	0x4013b2 <.text+0x3b2>
  4010b0: ff 05 58 20 40 00            	incl	0x402058
  4010b6: e8 3b 00 00 00               	calll	0x4010f6 <.text+0xf6>
  4010bb: fe 0d c8 21 40 00            	decb	0x4021c8
  4010c1: 75 8a                        	jne	0x40104d <.text+0x4d>
  4010c3: 6a 00                        	pushl	$0x0
  4010c5: e8 ee 02 00 00               	calll	0x4013b8 <.text+0x3b8>
  4010ca: 6a 01                        	pushl	$0x1
  4010cc: e8 e7 02 00 00               	calll	0x4013b8 <.text+0x3b8>
  4010d1: e8 e8 02 00 00               	calll	0x4013be <.text+0x3be>
  4010d6: 8b f0                        	movl	%eax, %esi
  4010d8: bf c8 21 40 00               	movl	$0x4021c8, %edi         # imm = 0x4021C8
  4010dd: eb 0f                        	jmp	0x4010ee <.text+0xee>
  4010df: e8 9d 00 00 00               	calll	0x401181 <.text+0x181>
  4010e4: 74 0f                        	je	0x4010f5 <.text+0xf5>
  4010e6: 3d a0 00 00 00               	cmpl	$0xa0, %eax
  4010eb: 77 08                        	ja	0x4010f5 <.text+0xf5>
  4010ed: aa                           	stosb	%al, %es:(%edi)
  4010ee: e8 79 00 00 00               	calll	0x40116c <.text+0x16c>
  4010f3: 73 ea                        	jae	0x4010df <.text+0xdf>
  4010f5: c3                           	retl
  4010f6: bf d0 22 40 00               	movl	$0x4022d0, %edi         # imm = 0x4022D0
  4010fb: be db 21 40 00               	movl	$0x4021db, %esi         # imm = 0x4021DB
  401100: 57                           	pushl	%edi
  401101: 8b 0d 70 20 40 00            	movl	0x402070, %ecx
  401107: b0 08                        	movb	$0x8, %al
  401109: f3 aa                        	rep		stosb	%al, %es:(%edi)
  40110b: 57                           	pushl	%edi
  40110c: e8 6a 00 00 00               	calll	0x40117b <.text+0x17b>
  401111: a1 58 20 40 00               	movl	0x402058, %eax
  401116: e8 38 00 00 00               	calll	0x401153 <.text+0x153>
  40111b: e8 5b 00 00 00               	calll	0x40117b <.text+0x17b>
  401120: b8 00 00 01 00               	movl	$0x10000, %eax          # imm = 0x10000
  401125: f7 25 58 20 40 00            	mull	0x402058
  40112b: e8 23 00 00 00               	calll	0x401153 <.text+0x153>
  401130: 58                           	popl	%eax
  401131: 2b c7                        	subl	%edi, %eax
  401133: f7 d8                        	negl	%eax
  401135: a3 70 20 40 00               	movl	%eax, 0x402070
  40113a: 58                           	popl	%eax
  40113b: 2b f8                        	subl	%eax, %edi
  40113d: 6a 00                        	pushl	$0x0
  40113f: ff 35 6c 20 40 00            	pushl	0x40206c
  401145: 57                           	pushl	%edi
  401146: 50                           	pushl	%eax
  401147: ff 35 68 20 40 00            	pushl	0x402068
  40114d: e8 72 02 00 00               	calll	0x4013c4 <.text+0x3c4>
  401152: c3                           	retl
  401153: b9 0a 00 00 00               	movl	$0xa, %ecx
  401158: 51                           	pushl	%ecx
  401159: 2b d2                        	subl	%edx, %edx
  40115b: f7 f1                        	divl	%ecx
  40115d: 52                           	pushl	%edx
  40115e: 85 c0                        	testl	%eax, %eax
  401160: 75 f7                        	jne	0x401159 <.text+0x159>
  401162: 58                           	popl	%eax
  401163: 0c 30                        	orb	$0x30, %al
  401165: aa                           	stosb	%al, %es:(%edi)
  401166: 58                           	popl	%eax
  401167: 3b c1                        	cmpl	%ecx, %eax
  401169: 72 f8                        	jb	0x401163 <.text+0x163>
  40116b: c3                           	retl
  40116c: ac                           	lodsb	(%esi), %al
  40116d: 3c 20                        	cmpb	$0x20, %al
  40116f: 77 fb                        	ja	0x40116c <.text+0x16c>
  401171: 72 05                        	jb	0x401178 <.text+0x178>
  401173: ac                           	lodsb	(%esi), %al
  401174: 3c 20                        	cmpb	$0x20, %al
  401176: 74 fb                        	je	0x401173 <.text+0x173>
  401178: 4e                           	decl	%esi
  401179: c3                           	retl
  40117a: aa                           	stosb	%al, %es:(%edi)
  40117b: ac                           	lodsb	(%esi), %al
  40117c: 3c 20                        	cmpb	$0x20, %al
  40117e: 73 fa                        	jae	0x40117a <.text+0x17a>
  401180: c3                           	retl
  401181: 2b d2                        	subl	%edx, %edx
  401183: 2b c0                        	subl	%eax, %eax
  401185: 8d 14 92                     	leal	(%edx,%edx,4), %edx
  401188: 8d 14 50                     	leal	(%eax,%edx,2), %edx
  40118b: ac                           	lodsb	(%esi), %al
  40118c: 2c 30                        	subb	$0x30, %al
  40118e: 3c 09                        	cmpb	$0x9, %al
  401190: 76 f3                        	jbe	0x401185 <.text+0x185>
  401192: 92                           	xchgl	%edx, %eax
  401193: 4e                           	decl	%esi
  401194: 85 c0                        	testl	%eax, %eax
  401196: c3                           	retl
  401197: be c4 21 40 00               	movl	$0x4021c4, %esi         # imm = 0x4021C4
  40119c: b9 0d 00 00 00               	movl	$0xd, %ecx
  4011a1: 8a 66 03                     	movb	0x3(%esi), %ah
  4011a4: c1 e8 02                     	shrl	$0x2, %eax
  4011a7: 8a 66 02                     	movb	0x2(%esi), %ah
  4011aa: c1 e8 02                     	shrl	$0x2, %eax
  4011ad: 8a 66 01                     	movb	0x1(%esi), %ah
  4011b0: c1 e8 02                     	shrl	$0x2, %eax
  4011b3: 8a 26                        	movb	(%esi), %ah
  4011b5: c1 e8 02                     	shrl	$0x2, %eax
  4011b8: 83 ee 04                     	subl	$0x4, %esi
  4011bb: aa                           	stosb	%al, %es:(%edi)
  4011bc: e2 e3                        	loop	0x4011a1 <.text+0x1a1>
  4011be: c3                           	retl
  4011bf: c1 e8 0e                     	shrl	$0xe, %eax
  4011c2: be 15 e8 c8 01               	movl	$0x1c8e815, %esi        # imm = 0x1C8E815
  4011c7: 40                           	incl	%eax
  4011c8: 8b d0                        	movl	%eax, %edx
  4011ca: 35 ff ff 00 00               	xorl	$0xffff, %eax           # imm = 0xFFFF
  4011cf: c1 c8 0a                     	rorl	$0xa, %eax
  4011d2: 8a e6                        	movb	%dh, %ah
  4011d4: 8a c2                        	movb	%dl, %al
  4011d6: c1 c8 0a                     	rorl	$0xa, %eax
  4011d9: c1 e2 02                     	shll	$0x2, %edx
  4011dc: 0a e6                        	orb	%dh, %ah
  4011de: 0a c2                        	orb	%dl, %al
  4011e0: 0c 03                        	orb	$0x3, %al
  4011e2: bf 00 20 40 00               	movl	$0x402000, %edi         # imm = 0x402000
  4011e7: b9 05 00 00 00               	movl	$0x5, %ecx
  4011ec: f7 ee                        	imull	%esi
  4011ee: 48                           	decl	%eax
  4011ef: ab                           	stosl	%eax, %es:(%edi)
  4011f0: e2 fa                        	loop	0x4011ec <.text+0x1ec>
  4011f2: bf 0c 00 00 00               	movl	$0xc, %edi
  4011f7: e8 fd 00 00 00               	calll	0x4012f9 <.text+0x2f9>
  4011fc: 4f                           	decl	%edi
  4011fd: 75 f8                        	jne	0x4011f7 <.text+0x1f7>
  4011ff: c3                           	retl
  401200: a1 54 20 40 00               	movl	0x402054, %eax
  401205: a9 ff 3f 00 00               	testl	$0x3fff, %eax           # imm = 0x3FFF
  40120a: 75 05                        	jne	0x401211 <.text+0x211>
  40120c: e8 ae ff ff ff               	calll	0x4011bf <.text+0x1bf>
  401211: bf 3c 20 40 00               	movl	$0x40203c, %edi         # imm = 0x40203C
  401216: e8 de 00 00 00               	calll	0x4012f9 <.text+0x2f9>
  40121b: 89 47 0c                     	movl	%eax, 0xc(%edi)
  40121e: ab                           	stosl	%eax, %es:(%edi)
  40121f: e8 d5 00 00 00               	calll	0x4012f9 <.text+0x2f9>
  401224: 89 47 0c                     	movl	%eax, 0xc(%edi)
  401227: ab                           	stosl	%eax, %es:(%edi)
  401228: e8 cc 00 00 00               	calll	0x4012f9 <.text+0x2f9>
  40122d: ba 16 e3 55 ad               	movl	$0xad55e316, %edx       # imm = 0xAD55E316
  401232: 81 7f fc 65 da 4d 63         	cmpl	$0x634dda65, -0x4(%edi) # imm = 0x634DDA65
  401239: 72 0c                        	jb	0x401247 <.text+0x247>
  40123b: 77 09                        	ja	0x401246 <.text+0x246>
  40123d: 81 7f f8 00 92 f4 8b         	cmpl	$0x8bf49200, -0x8(%edi) # imm = 0x8BF49200
  401244: 72 01                        	jb	0x401247 <.text+0x247>
  401246: 4a                           	decl	%edx
  401247: f7 e2                        	mull	%edx
  401249: 2b c9                        	subl	%ecx, %ecx
  40124b: 89 17                        	movl	%edx, (%edi)
  40124d: 89 57 0c                     	movl	%edx, 0xc(%edi)
  401250: c7 05 14 20 40 00 0d 0d 0d 0d	movl	$0xd0d0d0d, 0x402014    # imm = 0xD0D0D0D
  40125a: c7 05 18 20 40 00 03 02 01 00	movl	$0x10203, 0x402018      # imm = 0x10203
  401264: bd 34 00 00 00               	movl	$0x34, %ebp
  401269: be 5c 20 40 00               	movl	$0x40205c, %esi         # imm = 0x40205C
  40126e: bf 30 20 40 00               	movl	$0x402030, %edi         # imm = 0x402030
  401273: bb 17 20 40 00               	movl	$0x402017, %ebx         # imm = 0x402017
  401278: a5                           	movsl	(%esi), %es:(%edi)
  401279: a5                           	movsl	(%esi), %es:(%edi)
  40127a: a5                           	movsl	(%esi), %es:(%edi)
  40127b: be 24 20 40 00               	movl	$0x402024, %esi         # imm = 0x402024
  401280: e8 e9 00 00 00               	calll	0x40136e <.text+0x36e>
  401285: 8b 46 20                     	movl	0x20(%esi), %eax
  401288: 39 46 08                     	cmpl	%eax, 0x8(%esi)
  40128b: 72 13                        	jb	0x4012a0 <.text+0x2a0>
  40128d: 77 56                        	ja	0x4012e5 <.text+0x2e5>
  40128f: 8b 46 1c                     	movl	0x1c(%esi), %eax
  401292: 39 46 04                     	cmpl	%eax, 0x4(%esi)
  401295: 72 09                        	jb	0x4012a0 <.text+0x2a0>
  401297: 77 4c                        	ja	0x4012e5 <.text+0x2e5>
  401299: 8b 46 18                     	movl	0x18(%esi), %eax
  40129c: 39 06                        	cmpl	%eax, (%esi)
  40129e: 77 45                        	ja	0x4012e5 <.text+0x2e5>
  4012a0: e8 b7 00 00 00               	calll	0x40135c <.text+0x35c>
  4012a5: 8b 46 20                     	movl	0x20(%esi), %eax
  4012a8: 39 46 08                     	cmpl	%eax, 0x8(%esi)
  4012ab: 72 13                        	jb	0x4012c0 <.text+0x2c0>
  4012ad: 77 36                        	ja	0x4012e5 <.text+0x2e5>
  4012af: 8b 46 1c                     	movl	0x1c(%esi), %eax
  4012b2: 39 46 04                     	cmpl	%eax, 0x4(%esi)
  4012b5: 72 09                        	jb	0x4012c0 <.text+0x2c0>
  4012b7: 77 2c                        	ja	0x4012e5 <.text+0x2e5>
  4012b9: 8b 46 18                     	movl	0x18(%esi), %eax
  4012bc: 39 06                        	cmpl	%eax, (%esi)
  4012be: 77 25                        	ja	0x4012e5 <.text+0x2e5>
  4012c0: e8 97 00 00 00               	calll	0x40135c <.text+0x35c>
  4012c5: 8b 46 20                     	movl	0x20(%esi), %eax
  4012c8: 39 46 08                     	cmpl	%eax, 0x8(%esi)
  4012cb: 72 13                        	jb	0x4012e0 <.text+0x2e0>
  4012cd: 77 16                        	ja	0x4012e5 <.text+0x2e5>
  4012cf: 8b 46 1c                     	movl	0x1c(%esi), %eax
  4012d2: 39 46 04                     	cmpl	%eax, 0x4(%esi)
  4012d5: 72 09                        	jb	0x4012e0 <.text+0x2e0>
  4012d7: 77 0c                        	ja	0x4012e5 <.text+0x2e5>
  4012d9: 8b 46 18                     	movl	0x18(%esi), %eax
  4012dc: 39 06                        	cmpl	%eax, (%esi)
  4012de: 77 05                        	ja	0x4012e5 <.text+0x2e5>
  4012e0: e8 77 00 00 00               	calll	0x40135c <.text+0x35c>
  4012e5: 8a 43 04                     	movb	0x4(%ebx), %al
  4012e8: fe 0b                        	decb	(%ebx)
  4012ea: 88 85 93 21 40 00            	movb	%al, 0x402193(%ebp)
  4012f0: 4d                           	decl	%ebp
  4012f1: 0f 85 77 ff ff ff            	jne	0x40126e <.text+0x26e>
  4012f7: f8                           	clc
  4012f8: c3                           	retl
  4012f9: b8 c7 ff d4 7d               	movl	$0x7dd4ffc7, %eax       # imm = 0x7DD4FFC7
  4012fe: f7 25 0c 20 40 00            	mull	0x40200c
  401304: 8b f0                        	movl	%eax, %esi
  401306: a1 08 20 40 00               	movl	0x402008, %eax
  40130b: 8b ca                        	movl	%edx, %ecx
  40130d: ba d4 05 00 00               	movl	$0x5d4, %edx            # imm = 0x5D4
  401312: a3 0c 20 40 00               	movl	%eax, 0x40200c
  401317: f7 e2                        	mull	%edx
  401319: 03 f0                        	addl	%eax, %esi
  40131b: a1 04 20 40 00               	movl	0x402004, %eax
  401320: 13 ca                        	adcl	%edx, %ecx
  401322: ba f0 06 00 00               	movl	$0x6f0, %edx            # imm = 0x6F0
  401327: a3 08 20 40 00               	movl	%eax, 0x402008
  40132c: f7 e2                        	mull	%edx
  40132e: 03 f0                        	addl	%eax, %esi
  401330: a1 00 20 40 00               	movl	0x402000, %eax
  401335: 13 ca                        	adcl	%edx, %ecx
  401337: ba fb 13 00 00               	movl	$0x13fb, %edx           # imm = 0x13FB
  40133c: a3 04 20 40 00               	movl	%eax, 0x402004
  401341: f7 e2                        	mull	%edx
  401343: 03 c6                        	addl	%esi, %eax
  401345: 13 d1                        	adcl	%ecx, %edx
  401347: 03 05 10 20 40 00            	addl	0x402010, %eax
  40134d: 83 d2 00                     	adcl	$0x0, %edx
  401350: a3 00 20 40 00               	movl	%eax, 0x402000
  401355: 89 15 10 20 40 00            	movl	%edx, 0x402010
  40135b: c3                           	retl
  40135c: 4b                           	decl	%ebx
  40135d: 8b 06                        	movl	(%esi), %eax
  40135f: 8b 56 04                     	movl	0x4(%esi), %edx
  401362: 29 46 18                     	subl	%eax, 0x18(%esi)
  401365: 8b 46 08                     	movl	0x8(%esi), %eax
  401368: 19 56 1c                     	sbbl	%edx, 0x1c(%esi)
  40136b: 19 46 20                     	sbbl	%eax, 0x20(%esi)
  40136e: 8a 0b                        	movb	(%ebx), %cl
  401370: 8b 46 0c                     	movl	0xc(%esi), %eax
  401373: f7 e1                        	mull	%ecx
  401375: 50                           	pushl	%eax
  401376: 8b fa                        	movl	%edx, %edi
  401378: 8b 46 10                     	movl	0x10(%esi), %eax
  40137b: f7 e1                        	mull	%ecx
  40137d: 03 c7                        	addl	%edi, %eax
  40137f: 83 d2 00                     	adcl	$0x0, %edx
  401382: 50                           	pushl	%eax
  401383: 8b fa                        	movl	%edx, %edi
  401385: 8b 46 14                     	movl	0x14(%esi), %eax
  401388: f7 e1                        	mull	%ecx
  40138a: 03 c7                        	addl	%edi, %eax
  40138c: 83 d2 00                     	adcl	$0x0, %edx
  40138f: f7 f5                        	divl	%ebp
  401391: 89 46 08                     	movl	%eax, 0x8(%esi)
  401394: 58                           	popl	%eax
  401395: f7 f5                        	divl	%ebp
  401397: 89 46 04                     	movl	%eax, 0x4(%esi)
  40139a: 58                           	popl	%eax
  40139b: f7 f5                        	divl	%ebp
  40139d: 89 06                        	movl	%eax, (%esi)
  40139f: c3                           	retl
  4013a0: ff 25 38 22 40 00            	jmpl	*0x402238
  4013a6: ff 25 3c 22 40 00            	jmpl	*0x40223c
  4013ac: ff 25 40 22 40 00            	jmpl	*0x402240
  4013b2: ff 25 44 22 40 00            	jmpl	*0x402244
  4013b8: ff 25 48 22 40 00            	jmpl	*0x402248
  4013be: ff 25 4c 22 40 00            	jmpl	*0x40224c
  4013c4: ff 25 50 22 40 00            	jmpl	*0x402250
