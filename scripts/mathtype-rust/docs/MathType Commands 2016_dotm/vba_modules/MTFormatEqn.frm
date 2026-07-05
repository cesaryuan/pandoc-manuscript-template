Attribute VB_Name = "MTFormatEqn"
Attribute VB_Base = "0{CA5B6CED-DE3C-4884-A5F0-72CC5FD0C524}{FDA4B415-DAE2-4840-8092-973C268C8820}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False



'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTFormatEqn.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

Private Sub UserForm_Initialize()
    Dim tempPrefs$
    Dim myControl As MSForms.control
    Dim fontName$
    Dim fontSize As Long
    Dim Doc As Document

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!00118"))
    'Me.height = 5730
    'Me.width = 6750
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 223.25  '3945
    'Me.width = 347      '6750
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize
    
    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName$
        'myControl.font.size = fontSize
        If myControl.name <> "txt_FileName" Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl

    'default is to use doc's preferences
    'if there are no preferences in the document, grey out that option
    Set Doc = ActiveDocument
    If MTLib.DocContainsEquationSettings(Doc) Then
        opt_Document.value = True
    Else
        opt_Document.enabled = False
        Opt_MTDefault.value = True
    End If
    
    'check selection and set selection button accordingly
    'turn on 'Selection' button if there is one
    If Selection.Type = wdSelectionIP Then
        opt_rSelection.enabled = False
        opt_rWhole.value = True
    Else
        opt_rSelection.value = True
    End If
    
    tempPrefs$ = Strings.Space(128)
    'Fails to detect WMFs with MTEF after Word 2000.  See MT-1131
    If MTEquationOnClipboard() = mtNOT_EQUATION Then
        opt_EqOnClipboard.enabled = False
    End If

    MTFormatEquations.gDlgCanceled = True
End Sub

Private Sub btn_Browse_Click()
    Dim fileName$
    
    If MTLib.ChoosePrefFile(fileName$) Then
        txt_FileName = fileName$
    End If
    If Len(txt_FileName) > 0 Then
        opt_File.value = True
    End If
End Sub

'get the preferences and send to the Preview Preferences dialog
Private Sub btn_Preview_Click()
    Dim pref$
    Dim curWindow As Long

    #If Win32 Then
    curWindow = GetForegroundWindow()
    #End If
    
    'errors are displayed by the subroutines
    If opt_Document.value Then
        'Check if we're using MT's prefs or the prefs in the document
        If MTLib.DocUsesEquationSettings(ActiveDocument) Then
            pref$ = MTLib.GetPrefsFromDoc
        Else
            pref$ = MTLib.GetPrefsFromMType$
        End If
    ElseIf Opt_MTDefault.value Then
        pref$ = MTLib.GetPrefsFromMType$
    ElseIf opt_EqOnClipboard.value Then
        pref$ = MTLib.GetPrefsFromClipboard$
    ElseIf opt_File.value Then
        If txt_FileName.value = "" Then
            MsgBox MTLib.GetUserString("!0514You must choose a preference file."), vbExclamation, _
                MTLib.GetUserString("!0500Format Equations")
        Else
            'preview only if we can get info from the file
            'if error, message displayed by GetPrefsFromFile()
            pref$ = MTLib.GetPrefsFromFile$(txt_FileName.value)
        End If
    End If
    
    #If Win32 Then
    SetForegroundWindow curWindow
    #End If

    If pref$ <> "" Then
        MTLib.ShowPreviewDialog pref$, curWindow
    End If
End Sub

'Return the preference string. If any problems with string, return to dialog
Private Sub OK_Click()
    'contains source of preferences source upon return
    MTFormatEquations.gMTPrefsSource$ = ""

    'take down the dialog because GetPrefsFromClipboard$() will hang in Word 2000
    'for up to several minutes. The dialog is re-displayed if there are any errors
    Me.Hide
    
    If opt_Document.value Then
        MTFormatEquations.gMTPrefs$ = MTLib.GetPrefsFromDoc$
        If MTFormatEquations.gMTPrefs$ = "" Then GoTo repeatDlg
    ElseIf Opt_MTDefault.value Then
        MTFormatEquations.gMTPrefs$ = MTLib.GetPrefsFromMType$
        If MTFormatEquations.gMTPrefs$ = "" Then GoTo repeatDlg
        MTFormatEquations.gMTPrefsSource$ = MTLib.GetUserString("!0810MathType")
    ElseIf opt_EqOnClipboard.value Then
        MTFormatEquations.gMTPrefs$ = MTLib.GetPrefsFromClipboard$
        If MTFormatEquations.gMTPrefs$ = "" Then GoTo repeatDlg
        MTFormatEquations.gMTPrefsSource$ = MTLib.GetUserString("!0809clipboard")
    ElseIf opt_File.value Then
        If txt_FileName.value = "" Then
            MsgBox MTLib.GetUserString("!0514You must choose a preference file."), _
                vbExclamation, MTLib.GetUserString("!0500Format Equations")
            GoTo repeatDlg
        Else
            MTFormatEquations.gMTPrefs$ = MTLib.GetPrefsFromFile$(txt_FileName.value)
            If MTFormatEquations.gMTPrefs$ = "" Then GoTo repeatDlg
            MTFormatEquations.gMTPrefsSource$ = _
                MTLib.GetFileNameFromPath$(txt_FileName.value)
        End If
    End If
    
    'Return the range
    If opt_rWhole.value Then
        MTFormatEquations.gUpdateRange = mt_RANGE_DOCUMENT
    Else
        MTFormatEquations.gUpdateRange = mt_RANGE_SELECTION
    End If
    
    'save prefs only if checked and enabled
    MTFormatEquations.gSavePrefs = _
        (chk_SavePrefs.enabled And chk_SavePrefs.value)
    MTFormatEquations.gDlgCanceled = False
    unload Me
    Exit Sub
    
repeatDlg:
    'put the dialog back up
    Me.Show
End Sub

Private Sub Cancel_Click()
    MTFormatEquations.gDlgCanceled = True
    unload Me
End Sub

Private Sub btn_Help_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#format_equations_dialog"
'    MTLib.MTHelpTopic hlpMSWDFormat_Equations_Dialog
End Sub

Private Sub txt_FileName_Change()
    opt_File.value = True
End Sub
