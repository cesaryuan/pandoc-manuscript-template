Attribute VB_Name = "AutoExecCls"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False
'AutoExecCls
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/AutoExecCls.cls 26    10/11/11 2:12p Jimm $
'=====================================================================
Option Explicit

' Note: * all functions/subs that start with MTCommand_ are "top-level" entry points called from
'         OnAction handlers (e.g. buttons, menu items, etc).
'       * all functions/subs that start with MTCallback_ are are called by
'         Word 2007 as needed to refresh the Ribbon UI
'       * all functions/subs that start with NoDirectCall_
'          MUST NOT BE CALLED except via a RunXXX command

' module name
Private Const module As String = "AutoExecCls"

Private m_lError As Long
Private m_sError As String

' Expose the error description
' This error number is required because the CallByName
' function does not return the correct error
Public Property Get ErrorDescription() As String
    ErrorDescription = m_sError
End Property

' Expose the error number
' This error number is required because the CallByName
' function does not return the correct error
Public Property Get ErrorNumber() As Long
    ErrorNumber = m_lError
End Property

Public Sub NoDirectCall_Main()

    'If an error occurs in the logging initialization code, default to non-verbose logging
    On Error Resume Next

    Dim regValue As String
    regValue = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_VERBOSE_LOGGING_KEY)
    Select Case regValue
    Case "1"
        gVerboseLogging = True
    Case "0"
        gVerboseLogging = False
    Case Else
        gVerboseLogging = False
    End Select

    WriteLog "Word logging enabled"

    ' Replace logging init error handler with main error handler
    On Error GoTo fail


    'WriteLog "Storing Application in MTApp"
    'Set MTApp.App = Application
    'above class is intialized in Main() now, therefore calling UIUpdate specifically here
    UIUpdate

     'unload any MTW4 macros
    WriteLog "About to check MT4 macros"
    UnloadMT4Macros

    'check for MTW3 macros in the Normal template
    WriteLog "About to check for MTW3 macros in Normal template"
    If UILib.RemoveMT3Macros(NormalTemplate) Then
        WriteLog "Found MT3 macros"

        MsgBox UILib.GetUserString("!0201MathType has found and removed an old version of the MathType Commands for Word. The new commands will be available the next time you run Word. Word will now Exit."), _
            vbInformation, UILib.GetUserString("!0200MathType Commands")

        'make sure no MTW3 menu items remain...
        RemoveOldMenuItems

        WriteLog "removed MT3 macros - quitting"

        Application.Quit
    End If

    'initialize default side for equation numbering
    #If Win32 Then
    If Val(Application.version) >= kWord2007 Then
        WriteLog "About to initialize eqn numbering side"
        InitDefaultEqnNumSide
            WriteLog "Successfully initialized eqn numbering side"
        End If
    #Else
    WriteLog "About to initialize eqn numbering side"
    InitDefaultEqnNumSide
    WriteLog "Successfully initialized eqn numbering side"
    #End If

    #If Win32 Then
        WriteLog "About to call UIUpdate"
    UIUpdate
    If Val(Application.version) < kWord2007 Then
        If Not MTApp.App Is Nothing Then
            WriteLog "queuing UpdatePre2007CommandBar"
            MTApp.App.OnTime Now + TimeValue("00:00:01"), "UpdatePre2007CommandBar"
        End If
    End If
    #ElseIf 0 Then
    WriteLog "About to call UpdateState"
    UpdateState
    If Not MTApp.App Is Nothing Then
        ' http://valor:8080/browse/MT-2861
        WriteLog "Scheduling UpdatePre2007CommandBar before"
        On Error GoTo ScheduleErr1
        MTApp.App.OnTime Now + TimeValue("00:00:15"), "UpdatePre2007CommandBar"
        WriteLog "Scheduling UpdatePre2007CommandBar after"
        GoTo ScheduleComplete
ScheduleErr1:
        WriteLog "Scheduling Error - attempting immediate call of UpdatePre2007CommandBar"
        On Error GoTo ScheduleComplete
        UpdatePre2007CommandBar
        WriteLog "Returned from UpdatePre2007CommandBar immediate call"
ScheduleComplete:
    Else
        WriteLog "Scheduling UpdatePre2007CommandBar was skipped - MTApp.App is nothing"
    End If
    #End If

        'Update WordVer usage statistic
    Dim wordMajorVersion As Long
    wordMajorVersion = Int(Val(Application.version))
    MTSetStatistic "WordVer", wordMajorVersion

        'Install our keyboard shortcuts
    AddKeyboardShortcuts

    WriteLog "AutoExecCls.NoDirectCall_Main success"
    Exit Sub

fail:
    ' First, pass along any DSI_ABORT_EXCEPTIONs to be caught by the top-level handler
    If err.Number = DSI_ABORT_EXCEPTION Then
        err.Raise DSI_ABORT_EXCEPTION 'leaves other Err data intact
    Else
        ' handle unexpected error by displaying message, log error, throw DSI private error
        Asserts.AssertFailure module, "Main", err.Description
    End If

    Exit Sub

unload:
    'can't display error as WLL is missing, so continue quietly
    AddIns(MacroContainer).Installed = False

        WriteLog "Unloaded ourselves, and exiting quietly"
End Sub

'Unload MathType4 macros if found
Private Sub UnloadMT4Macros()
    Dim addin As addin
    Dim prefix As String
    Dim prefixLen As Long

    prefix = "MathType Commands For Word"
    prefixLen = Len(prefix)
    On Error GoTo errHandler
    For Each addin In AddIns

            WriteLog "Considering addin " & addin

        'XP has a 'bug' where asking the name for an unloaded & non-existent AddIn throws an error
        If left$(addin.name, prefixLen) = prefix Then

                WriteLog "matched prefix"

            MsgBox UILib.GetUserString("!0203An older version of the MathType Commands for Word is installed and will now be removed. Please ignore the ensuing message (Word cannot open this document template)."), _
                vbInformation, UILib.GetUserString("!0200MathType Commands")
            addin.Installed = False

                WriteLog "About to Kill addin"
            KillAddIn addin.path, Application.PathSeparator, addin.name

                WriteLog "About to Delete addin"
            addin.delete

                WriteLog "exiting prefix match conditional"
            End If
continue:
    Next
    Exit Sub

errHandler:
        WriteLog "in UnloadMT4Macros errhandler looking at error number " & err.Number
    Resume continue

End Sub

Private Sub KillAddIn(path As String, separator As String, name As String)
    'wrap Kill in error handler as it throws an exception if file is busy/missing
    On Error Resume Next
    FileSystem.Kill path & separator & name
End Sub

'Remove MTW3 menu items from Normal template
'Search all menus in the menubar to avoid using explicit menu names
Private Sub RemoveOldMenuItems()
    Dim curMenu As CommandBarControl
    Dim curItem As CommandBarControl

    CustomizationContext = NormalTemplate

        WriteLog "About to interate through menubar controls"

    For Each curMenu In CommandBars.ActiveMenuBar.Controls
        For Each curItem In curMenu.Controls
                WriteLog "considering" & curItem.caption
            Select Case curItem.caption
            Case "Equation Number", "Equation Reference", "Equation Section", _
                "Equation Section...", "Equation Number Format...", "MathType Preferences", _
                "MathType Preferences...", "Update Equations", "Update Equations..."

                curItem.delete
                    WriteLog "deleted" & curItem.caption
            End Select
        Next curItem
    Next curMenu

        WriteLog "exiting RemoveOldMenuItems"
End Sub

'Adds our keyboard shortcuts
'Inline Eqn                         = Ctrl+Alt+Q = 512 + 1024 + 81 = 1617
    'Mac                            = Ctrl+Option+Q         = 4096 + 2048 + 81 = 6225

'Display Eqn                        = Alt+Q = 1024 + 81 = 1105
    'Mac                            = Option+Q              = 2048 + 81 = 2129

'Right Numbered Disp Eqn            = Shift+Alt+Q = 256 + 1024 + 81 = 1361
    'Mac                            = Shift+Option+Q        = 512 + 2048 + 81 = 2641

'Left Numbered Disp Eqn             = Ctrl+Shift+Alt+Q = 512 + 256 + 1024 + 81 = 1873
    'Mac                            = Ctrl+Shift+Option+Q   = 4096 + 512 + 2048 + 81 = 6737

'Tex Toggle                         = Alt+\ = 1024 + 220 = 1244
    'Mac                            = n/a                   = 2048 + 220 = 2268
    'Mac Alternate                  = Ctrl+X                = 4096 + 88 = 4184

'Edit in place selected equation    = Alt+E = 1024 + 69 = 1093
    'Mac                            = n/a                   = 2048 + 69 = 2117 - For reference only, not supported on the Mac

'Edit in MT selected equation       = Alt+O = 1024 + 79 = 1103
    'Mac                            = Option+Command+O      = 2048 + 256 + 79 = 2383

'Open Math Input Panel              = Ctrl+Shift+M = 512 + 256 + 77 = 845
    'Mac                            = n/a                   = 4096 + 512 + 77 = 4685 - For reference only, not supported on the Mac

Public Sub AddKeyboardShortcuts()

    On Error Resume Next

    #If Win32 Then
    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertInlineEqn", KeyCode:=Val(UILib.GetUserString("!24001617"))

    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertDispEqn", KeyCode:=Val(UILib.GetUserString("!24011105"))

    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertRightNumberedDispEqn", KeyCode:=Val(UILib.GetUserString("!24021361"))

    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertLeftNumberedDispEqn", KeyCode:=Val(UILib.GetUserString("!24031873"))

    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_TeXToggle", KeyCode:=Val(UILib.GetUserString("!24041244"))

    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_EditEquationInPlace", KeyCode:=Val(UILib.GetUserString("!24051093"))

    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_EditEquationOpen", KeyCode:=Val(UILib.GetUserString("!24061103"))

    If IsMathInputPanelAvailable Then
        KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_MathInputControl", KeyCode:=Val(UILib.GetUserString("!24070845"))
    End If

    #Else
    WriteLog "AddKeyShortcut: InsertInlineEqn"
    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertInlineEqn", KeyCode:=Val(UILib.GetUserString("!33246225"))

    WriteLog "AddKeyShortcut: InsertDispEqn"
    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertDispEqn", KeyCode:=Val(UILib.GetUserString("!33252129"))

    WriteLog "AddKeyShortcut: InsertRightNumberedDispEqn"
    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertRightNumberedDispEqn", KeyCode:=Val(UILib.GetUserString("!33262641"))

    WriteLog "AddKeyShortcut: InsertLeftNumberedDispEqn"
    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_InsertLeftNumberedDispEqn", KeyCode:=Val(UILib.GetUserString("!33276737"))

    WriteLog "AddKeyShortcut: TeXToggle"
    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_TeXToggle", KeyCode:=Val(UILib.GetUserString("!33282268"))
    If err.Number = 5346 Then
        WriteLog "Using alternate keystroke (Ctrl-X) for TexToggle"
        KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_TeXToggle", KeyCode:=Val(UILib.GetUserString("!33354184"))
    End If

    WriteLog "AddKeyShortcut: EditEquationOpen"
    KeyBindings.Add KeyCategory:=wdKeyCategoryMacro, command:="MTCommand_EditEquationOpen", KeyCode:=Val(UILib.GetUserString("!33302383"))

    WriteLog "AddKeyShortcut: Finished"
    #End If

    'If the Word option "Prompt before saving Normal template" is checked on Word always
    'prompts to save normal.dot because of the above KeyBindings additions.
    'See http://valor:8080/browse/MT-2176
    'To prevent this prompt we clear the template dirty flag
    Application.NormalTemplate.saved = True

    Exit Sub

err:
    MsgBox "AddKeyShortcut: error " & err.Number & Chr(13) & err.Description
    Resume Next

End Sub

