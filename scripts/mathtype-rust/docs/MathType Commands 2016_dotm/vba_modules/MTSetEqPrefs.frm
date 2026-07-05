Attribute VB_Name = "MTSetEqPrefs"
Attribute VB_Base = "0{D24A3496-131C-400E-B9C3-434AC60BD1E8}{32812602-5006-4A34-A4FD-3E122600CFAA}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False



'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTSetEqPrefs.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit
Dim gPrefFile$          'name of pref file from which settings obtained
Dim gPrefFileSettings$  'settings from pref file gPrefFile$

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim prefFile$
    Dim fontName$
    Dim fontSize As Long
    Dim Doc As Document
    
    'assign dialog's font & size for current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!00118"))
    'Me.height = 3015
    'Me.width = 8115
    '#Else
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 165.5   '2784
    'Me.width = 332      '6444
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize
    
    'assign captions
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName$
        'myControl.font.size = fontSize
        If myControl.name <> "lbl_Prefs" Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl

    Set Doc = ActiveDocument
    'set label for doc's preferences source
    gPrefFile$ = ""
    If MTLib.DocContainsEquationSettings(Doc) Then
        gPrefFile$ = MTLib.ReadDocPropString$(Doc, mtprop_PREFERENCES_FILE)
        If gPrefFile$ = "" Then
            gPrefFile$ = MTLib.GetUserString2("0808", "2408", "<unknown file>")
        End If
        lbl_Prefs.caption = MTLib.GetUserString2("0806", "2406", "(loaded from ") + gPrefFile$ + MTLib.GetUserString2("0811", "2411", ")")
    Else
        lbl_Prefs.caption = MTLib.GetUserString2("0805", "2405", "(no preferences have been loaded yet)")
    End If
    'read existing doc. settings
    gPrefFileSettings$ = MTLib.ReadDocPropString$(Doc, mtprop_PREFERENCES)
    
    'set radio buttons (has side effect of setting Preview button)
    If MTLib.DocUsesEquationSettings(Doc) Then
        Opt_DocSettings.value = True
    Else
        Opt_MTDefault.value = True
    End If
    
    MTSetEqnPrefs.gDlgCanceled = True
End Sub
Private Sub Opt_DocSettings_Click()
    btn_Preview.enabled = (gPrefFile$ <> "")
End Sub

Private Sub Opt_MTDefault_Click()
    btn_Preview.enabled = True
End Sub
Private Sub btn_Browse_Click()
    Dim fileName$
    
    'force "Doc Settings" on
    Opt_DocSettings.value = 1

    If MTLib.ChoosePrefFile(fileName$) Then
        gPrefFileSettings$ = MTLib.GetPrefsFromFile$(fileName$)
        If gPrefFileSettings$ <> "" Then
            gPrefFile$ = MTLib.GetFileNameFromPath$(fileName$)
            lbl_Prefs.caption = MTLib.GetUserString2("0806", "2406", "(loaded from ") + gPrefFile$ + MTLib.GetUserString2("0811", "2411", "!0811)")
            btn_Preview.enabled = True
        End If
    End If
End Sub
Private Sub btn_Help_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#set_equation_preferences_dialog"
'    MTLib.MTHelpTopic hlpMSWDSet_Equation_Preferences_Dialog
End Sub
Private Sub btn_Preview_Click()
    Dim pref$
    Dim curWindow As Long

    #If Win32 Then
    'save current window as we'll force ourself to front below
    curWindow = GetForegroundWindow()
    #End If
    
    'get the preferences and send to the Preview Preferences dialog
    'errors displayed by MTLib... subroutines
    If Opt_MTDefault.value Then
        pref$ = MTLib.GetPrefsFromMType$
    ElseIf Opt_DocSettings.value Then
        pref$ = gPrefFileSettings$
    End If
    
    #If Win32 Then
    SetForegroundWindow curWindow
    #End If

    If pref$ <> "" Then
        MTLib.ShowPreviewDialog pref$, curWindow
    End If
End Sub

Private Sub Cancel_Click()
    MTSetEqnPrefs.gDlgCanceled = True
    unload Me
End Sub

Private Sub OK_Click()
    If Opt_MTDefault.value Then
        gUseMTPrefs = True
    ElseIf Opt_DocSettings.value Then
        'If user hasn't loaded any prefs. yet, treat this like a Load... click
        If gPrefFile$ = "" Then
            btn_Browse_Click
            If gPrefFile$ <> "" And btn_Preview.enabled = False Then
                btn_Preview.enabled = True
            End If
            GoTo repeatDlg
        End If
        MTSetEqnPrefs.gMTPrefs$ = gPrefFileSettings$
        gUseMTPrefs = False
    End If
    
    'return doc. settings no matter which radio button selected
    MTSetEqnPrefs.gMTPrefFile$ = gPrefFile$
    MTSetEqnPrefs.gMTPrefs$ = gPrefFileSettings$
    
    MTSetEqnPrefs.gDlgCanceled = False
    unload Me
    
repeatDlg:
End Sub
