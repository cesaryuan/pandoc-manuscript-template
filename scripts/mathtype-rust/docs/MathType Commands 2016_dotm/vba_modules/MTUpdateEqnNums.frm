Attribute VB_Name = "MTUpdateEqnNums"
Attribute VB_Base = "0{09B106FB-3276-4D45-9919-ACA0E7FE4038}{FCE8885D-139A-47DF-BF96-7C6AF9EFFE00}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False


'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTUpdateEqnNums.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

Private Sub btnOK_Click()
    'if 'Don't Show' checked, write to registry
    If cbDontShowAgain.value Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_SLOWEQNUPDATE, "1"
    End If

    unload Me
End Sub

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim fontName As String
    Dim fontSize As Long

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize
    
    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName
        'myControl.font.size = fontSize
        myControl.caption = MTLib.GetUserString(myControl.caption)
                #If Mac Then
                If TypeOf myControl Is MSForms.Frame Then
                        myControl.BackColor = &H8000000F
                End If
                #End If
    Next

    cbDontShowAgain.value = False
End Sub

