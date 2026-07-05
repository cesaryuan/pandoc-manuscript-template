Attribute VB_Name = "MTOptionsDlg"
Attribute VB_Base = "0{8AA62FBA-9CCC-49A2-B5BB-A911008515B4}{488BA4FE-6D82-48E0-B11C-1BBE0770E54E}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False




'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTOptionsDlg.frm 1     5/06/14 9:55a Jimm $
'=====================================================================

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim settings
    Dim fontName$
    Dim fontSize As Long

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    'Me.height = 3360
    'Me.width = 7320
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 128.25  '2145
    'Me.width = 260.25   '5115
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

    'initialize controls
    If Val(Application.version) >= kWord2007 Then
        Dim mmlPaste
        mmlPaste = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS)
        If Len(mmlPaste) > 0 Then
            If mmlPaste = "mt" Then
                opt_MTEqn.value = True
            ElseIf mmlPaste = "omml" Then
                opt_OMMLEqn.value = True
            Else
                opt_Ask.value = True
            End If
        Else 'default to checked if it isn't set
            opt_MTEqn.value = True
        End If
    Else 'always paste MathML as MT equations when no OMML support
        opt_MTEqn.value = True
    End If

    Application.Activate 'in case MT had to be started up, grab the focus back
End Sub

Private Sub Help_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#mathtype_options_dialog"
'    MTLib.MTHelpTopic hlpMSWDOptions_MTOptions_Dialog
End Sub

Private Sub Cancel_Click()
    unload Me
End Sub

Private Sub OK_Click()
    ' process checkbox
    If opt_MTEqn.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS, "mt"
    ElseIf opt_OMMLEqn.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS, "omml"
    Else
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_ASK_ON_PASTE, 1
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS, "ask"
    End If

    If cb_Reset.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNNUM_WARNING, 0
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_SLOWEQNUPDATE, 0
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNREFDLG, 0
    End If
    unload Me
End Sub

