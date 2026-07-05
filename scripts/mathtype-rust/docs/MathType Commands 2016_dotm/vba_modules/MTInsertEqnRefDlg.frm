Attribute VB_Name = "MTInsertEqnRefDlg"
Attribute VB_Base = "0{A40D5767-2D03-4FBA-A8CA-F9638C3A1F19}{D3C8DD98-D63A-444C-9F0D-B2F3E50FA524}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False



'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTInsertEqnRefDlg.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

Private Sub btn_Help_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#insert_equation_reference_dialog"
'    MTLib.MTHelpTopic hlpMSWDInsert_Requation_Ref_Dialog
End Sub

Private Sub Cancel_Click()
    MTMarkRef.gDlgCanceled = True
    unload Me
End Sub

Private Sub OK_Click()
    If cbDontShowAgain.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNREFDLG, "1"
    End If
    MTMarkRef.gDlgCanceled = False
    unload Me
End Sub

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim fontName$
    Dim fontSize As Long

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    'Me.height = 1590
    'Me.width = 7695
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 105.5   '1590
    'Me.width = 308.75   '5985
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize

    'assign the captions
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName$
        'myControl.font.size = fontSize
        myControl.caption = MTLib.GetUserString(myControl.caption)
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl
    MTMarkRef.gDlgCanceled = True
End Sub

