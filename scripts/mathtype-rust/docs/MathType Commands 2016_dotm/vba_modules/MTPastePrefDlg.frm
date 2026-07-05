Attribute VB_Name = "MTPastePrefDlg"
Attribute VB_Base = "0{71FB3D32-7954-4D76-8673-F060D30AEC9A}{C244C061-384B-45DA-BEEE-25997C0BE37C}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False




'MFPastePrefDlg
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTPastePrefDlg.frm 1     5/06/14 9:55a Jimm $\
'=====================================================================

Private Sub UserForm_Initialize()
    Dim fontSize As Long
        Dim fontName As String
        Dim myControl As MSForms.control

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!00118"))
    'Me.height = 3840
    'Me.width = 5385
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 153.5   '2550
    'Me.width = 272.75   '5265
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize

    'assign the captions according to the current language
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

    Dim mmlPaste
    mmlPaste = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS)
    If mmlPaste = "omml" Then
        opt_MTEqn.value = False
        opt_OMMLEqn.value = True
    Else
        opt_MTEqn.value = True
        opt_OMMLEqn.value = False
    End If
    cb_Remember.value = False

    #If Win32 Then
    If Val(Application.version) >= kWord2007 Then
        lbl_Options.caption = MTLib.GetUserString2("3312", "3281", "Use the Options control on the MathType ribbon tab to change this option.")
    Else
        lbl_Options.caption = MTLib.GetUserString2("3311", "3282", "Use the Options command on the MathType menu to change this option.")
    End If
    #End If

    MTLib.gPastePrefDlgCanceled = True
    Application.Activate 'in case MT had to be started up, grab the focus back
End Sub

Private Sub btn_Help_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#mathtype_paste_pref_dialog"
'    MTLib.MTHelpTopic hlpMSWDPaste_Pref_Dialog
End Sub

Private Sub btnOK_Click()
    If opt_MTEqn.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS, "mt"
    ElseIf opt_OMMLEqn.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS, "omml"
    End If

    If cb_Remember.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_ASK_ON_PASTE, 0
    Else
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_ASK_ON_PASTE, 1
    End If

    MTLib.gPastePrefDlgCanceled = False
    unload Me
End Sub

Private Sub btnCancel_Click()
    MTLib.gPastePrefDlgCanceled = True
    unload Me
End Sub
