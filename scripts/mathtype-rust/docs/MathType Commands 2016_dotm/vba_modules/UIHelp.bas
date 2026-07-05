Attribute VB_Name = "UIHelp"
'====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UIHelp.bas 25    9/17/12 1:21p Jimm $
'====================================================================
' This module is dedicated to help related functionality
' It is shared between word and powerpoint

' Note: * all functions/subs that start with MTCommand_ are "top-level" entry points called from
'         OnAction handlers (e.g. buttons, menu items, etc).
'       * all functions/subs that start with MTCallback_ are are called by
'         Word 2007 as needed to refresh the Ribbon UI
'       * all functions/subs that start with NoDirectCall_
'          MUST NOT BE CALLED except via a RunXXX command

'URL codes - also see MTDeclarations.bas mturl's
Public Const mturlMATHTYPE_HOME As Long = 1
Public Const mturlMATHTYPE_SUPPORT As Long = 2
Public Const mturlMATHTYPE_FEEDBACK As Long = 3
Public Const mturlMATHTYPE_ORDER As Long = 4
Public Const mturlMATHTYPE_FUTURE As Long = 5
Public Const mturlMATHTYPE_REGISTER As Long = 6

'Constants for use in Help calls from dialogs
Public Const hlpMSWDMathType_Support_For_Word = 101

#If Win32 Then
Public Const hlpMSWDMathType_Support_For_Word_2007 = 102
#Else
Public Const hlpMSWDMathType_Support_For_Word_2007 = 3200
#End If

Public Const hlpMSWDMathType_Support_For_PP = 1

#If Win32 Then
Public Const hlpMSWDMathType_Support_For_PP_2007 = 3
#Else
Public Const hlpMSWDMathType_Support_For_PP_2007 = 3202
#End If

#If Win32 Then
Public Const hlpMSWDUnlock_MathType = 165
#Else
Public Const hlpMSWDUnlock_MathType = 195
#End If

Private Const module As String = "UIHelp"
Private Const moduleCLS As String = "UIHelpCls"
Public Sub OpenUrl(url As String)
    ShellExecute 0, "Open", url
End Sub
'Runs MathType Help
Public Sub MTCommand_ShowHelpContents()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowHelpContents", True, mtbIDHelpContents
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowHelpContents", True
#End If
End Sub

Public Sub MTCommand_ShowHelpMTInWord()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowHelpMTInWord", True, mtbIDHelpMTInWord
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowHelpMTInWord", True
#End If
End Sub

Public Sub MTCommand_ShowUnlockReg()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowUnlockReg", True, mtbIDHelpUnlockReg
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowUnlockReg", True
#End If
End Sub

' *** web help code ***

Public Sub MTCommand_ShowWebHomePage()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebHomePage", True, mtbIDWebHomePage
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebHomePage", True
#End If
End Sub

Public Sub MTCommand_ShowWebSupport()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebSupport", True, mtbIDWebSupport
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebSupport", True
#End If
End Sub

Public Sub MTCommand_ShowWebEmailFeedback()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebEmailFeedback", True, mtbIDWebEmailFeedback
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebEmailFeedback", True
#End If
End Sub

Public Sub MTCommand_ShowWebOrderMathType()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebOrderMathType", True, mtbIDWebOrderMT
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowWebOrderMathType", True
#End If
End Sub

Public Sub MTCommand_ShowFutureMT()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowFutureMT", True, mtbIDFutureMT
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowFutureMT", True
#End If
End Sub

' *** misc help code ***

Public Sub MTCommand_ShowAboutMT()
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowAboutMT", True, mtbIDHelpAboutMT
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowAboutMT", True
#End If
End Sub

' called from UIEnableDisable and UIRibbon
Public Function NoDirectCall_GetUnlockUI() As String
    ' note: we changed the UI to combine unlock and registration
    ' we are keeping the old code (following this line) around just in case we want to switch back
    NoDirectCall_GetUnlockUI = UILib.GetUserString2("3103", "3104", "U&nlock/Register MathType...")

    'Dim result As Long
    'result = MTGetAppFunctionality
    'If result = DemoMode.dmFull Then
    '    NoDirectCall_GetUnlockUI = UILib.GetUserString("!1896O&nline Registration...")
    'Else
    '    NoDirectCall_GetUnlockUI = UILib.GetUserString("!1838U&nlock MathType...")
    'End If

    ' Office 12 and greater does not need a keyboard mnemonic
    If Val(Application.version) >= kWord2007 Then
        #If Mac And PP Then
        NoDirectCall_GetUnlockUI = ReplaceSubstring(NoDirectCall_GetUnlockUI, "&", "")
        #Else
        NoDirectCall_GetUnlockUI = replace(NoDirectCall_GetUnlockUI, "&", "")
        #End If
    End If
End Function

Public Function NoDirectCall_GetUnlockScreenTipUI() As String
    ' note: we changed the UI to combine unlock and registration
    ' we are keeping the old code (following this line) around just in case we want to switch back
    NoDirectCall_GetUnlockScreenTipUI = UILib.GetUserString("!3102Unlock/Register MathType")

    'Dim result As Long
    'result = MTGetAppFunctionality
    'If result = DemoMode.dmFull Then
    '    NoDirectCall_GetUnlockScreenTipUI = UILib.GetUserString("!3101Register MathType")
    'Else
    '    NoDirectCall_GetUnlockScreenTipUI = UILib.GetUserString("!3100Unlock MathType")
    'End If
End Function


