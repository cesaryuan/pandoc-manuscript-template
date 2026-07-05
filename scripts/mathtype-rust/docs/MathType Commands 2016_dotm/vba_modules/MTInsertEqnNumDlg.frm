Attribute VB_Name = "MTInsertEqnNumDlg"
Attribute VB_Base = "0{AEB7C08E-0E9C-4525-A886-84EEAA3C1DF7}{99F68E2B-8744-4345-AF14-50196FDD202B}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False



'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTInsertEqnNumDlg.frm 1     5/06/14 9:55a Jimm $
'=====================================================================

Option Explicit

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim fontName$
    Dim fontSize As Long

    'assign dialog's font & size for current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!00118"))
    'Me.height = 3915
    'Me.width = 7980
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 146.5   '2310
    'Me.width = 349.75   '6705
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize

    'assign the captions
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName$
        'myControl.font.size = fontSize
        'update controls except edit fields
        If left$(myControl.name, 3) <> "edt" Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
    Next myControl

    MTEqnNum.gDlgCanceled = True
    MTEqnNum.gDontShowEqnNumWarning = False
    
    'needed to cause initial selection of contents
    edtChapterNumber.selStart = 0
    edtChapterNumber.SelLength = Len(edtChapterNumber)
End Sub

Private Sub OK_Click()
    Dim stat As Boolean
    
    'validate the entered numbers...
    stat = MTLib.IsValidExplicitNumber(edtSectionNumber, MTEqnNum.gSectionNumber, Me.caption)
    If stat Then
        stat = MTLib.IsValidExplicitNumber(edtChapterNumber, MTEqnNum.gChapterNumber, Me.caption)
    End If
    If stat Then
        MTEqnNum.gDontShowEqnNumWarning = cbDontShow
        MTEqnNum.gDlgCanceled = False
        unload Me
    End If
End Sub

Private Sub Cancel_Click()
    MTEqnNum.gDlgCanceled = True
    unload Me
End Sub

Private Sub btn_Help_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#insert_equation_number_dialog"
'    MTLib.MTHelpTopic hlpMSWDInsert_Equation_Number_Dialog
End Sub

