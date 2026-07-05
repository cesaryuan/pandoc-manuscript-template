Attribute VB_Name = "UIRibbon"
'UIRibbon 5.3
'====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UIRibbon.bas 57    6/23/10 6:50p Robertm $
'====================================================================

' This file is shared between Word and PowerPoint. Some functions are only
' intended to be used by Word at this time. Since a user might try to execute
' one of these macros, we will do a application name check to limit their execution.

Option Explicit

Public gMTBrowseType As Integer
Public gMTRibbon As IRibbonUI

Private Const module As String = "UIRibbon"
Private Const moduleCLS As String = "UIRibbonCls"

'boolean containing the value if user is informed about unsupported version of MS Word
Private UserInformed As Boolean

'called when ribbon loads; save ribbon to invoke its Invalidate method later
Public Sub MTCommand_OnRibbonLoaded(ribbon As IRibbonUI)
    Set gMTRibbon = ribbon
End Sub

'call back for setting the initial state of the browse dropdown
Public Sub MTCommand_OnSelectedIndexBrowseType(control As IRibbonControl, ByRef index)
    'RunDocCallback moduleCLS, "NoDirectCall_OnSelectedIndexBrowseType"
End Sub

'Runs Insert Inline Equation command
Public Sub MTCommand_OnInsertInlineEqn(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertInlineEqn", True, mtbIDInsInlineMTEqn
End Sub

'Runs Insert Display Equation command
Public Sub MTCommand_OnInsertDispEqn(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertDispEqn", True, mtbIDInsDispMTEqn
End Sub

'Runs Insert Left-Numbered Display Equation command
Public Sub MTCommand_OnInsertLeftNumberedDispEqn(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertLeftNumberedDispEqn", True, mtbIDInsDispMTEqnLeftNum
End Sub

'Runs Insert Right-Numbered Display Equation command
Public Sub MTCommand_OnInsertRightNumberedDispEqn(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertRightNumberedDispEqn", True, mtbIDInsDispMTEqnRightNum
End Sub

'Runs Insert Handwritten Math command
Public Sub MTCommand_OnInsHandEqn(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsHandEqn", True, mtbIDInsHandEqn
End Sub

'Inserts EB Inline Equation
Public Sub MTCommand_OnInsertInlineEqnEB(control As IRibbonControl, cancelDefault)
    'RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertInlineEqnEB", True, mtbIDMathType_B_InsEBEqn
End Sub

'Inserts EB Left-Numbered Display Equation
Public Sub MTCommand_OnInsertLeftNumberedDispEqnEB(control As IRibbonControl)
    'RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertLeftNumberedDispEqnEB", True, mtbIDInsEBEqnLeftNum
End Sub

'Inserts EB Right-Numbered Display Equation
Public Sub MTCommand_OnInsertRightNumberedDispEqnEB(control As IRibbonControl)
    'RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertRightNumberedDispEqnEB", True, mtbIDInsEBEqnRightNum
End Sub

'Runs Format Equations command
Public Sub MTCommand_OnFormatEquations(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnFormatEquations", True, mtbIDFormatEqns
End Sub

'Runs Convert Equations command
Public Sub MTCommand_OnConvertEquations(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnConvertEquations", True, mtbIDConvertEqns
End Sub

'Runs TeX Toggle command
Public Sub MTCommand_OnTeXToggle(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnTeXToggle", True, mtbIDTeXToggle
End Sub

'Runs Export Equations command
Public Sub MTCommand_OnExportEquations(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnExportEquations", True, mtbIDExportEqns
End Sub


'Runs MathPage command
Public Sub MTCommand_OnMathPage(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnMathPage", True, mtbIDMathPage
End Sub

'Runs Set Equation Prefs command
Public Sub MTCommand_OnSetEquationPrefs(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnSetEquationPrefs", True, mtbIDSetEqnPrefs
End Sub

'Runs Insert Equation Number command
Public Sub MTCommand_OnInsertEquationNumber(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertEquationNumber", True, mtbIDInsertNumber
End Sub

'Runs Format Equation Numbers command
Public Sub MTCommand_OnFormatEquationNumbers(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnFormatEquationNumbers", True, mtbIDFormatEqnNums
End Sub

'Runs Update Equation Numbers command
Public Sub MTCommand_OnUpdateEquationNumbers(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnUpdateEquationNumbers", True, mtbIDUpdateEqnNums
End Sub

'Runs Insert Equation Reference command
Public Sub MTCommand_OnEquationReference(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnEquationReference", True, mtbIDEquationReference
End Sub

'Runs Insert Chapter/Section Break command
Public Sub MTCommand_OnInsertChapterSectionBreak(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertChapterSectionBreak", True, mtbIDMoreBreaks
End Sub

'Runs Insert Chapter/Section Break command
Public Sub MTCommand_OnInsertNextSection(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertNextSection", True, mtbIDInsertNextSection
End Sub


'Runs Insert Chapter/Section Break command
Public Sub MTCommand_OnInsertNextChapter(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnInsertNextChapter", True, mtbIDInsertNextChapter
End Sub

'Runs Modify Chapter/Section Break command
Public Sub MTCommand_OnModifyChapterSectionBreak(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnModifyChapterSectionBreak", True, mtbIDModifyBreak
End Sub

'Runs MathType Help
Public Sub MTCommand_OnHelpContents(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnHelpContents", True, mtbIDHelp
End Sub

Public Sub MTCommand_OnHelpMTInWord(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnHelpMTInWord", True, mtbIDHelpMTInWord
End Sub

Public Sub MTCommand_OnUnlockReg(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnUnlockReg", True, mtbIDHelpUnlockReg
End Sub

Public Sub MTCommand_OnAboutMT(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnAboutMT", True, mtbIDHelpAboutMT
End Sub

Public Sub MTCommand_OnWebHomePage(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnWebHomePage", True, mtbIDWeb
End Sub

Public Sub MTCommand_OnWebSupport(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnWebSupport", True, mtbIDWebSupport

End Sub

Public Sub MTCommand_OnWebEmailFeedback(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnWebEmailFeedback", True, mtbIDWebEmailFeedback
End Sub

Public Sub MTCommand_OnWebOrderMathType(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnWebOrderMathType", True, mtbIDWebOrderMT
End Sub

Public Sub MTCommand_OnFutureMT(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnFutureMT", True, mtbIDFutureMT
End Sub

Public Sub MTCommand_OnMTOptions(control As IRibbonControl)
#If Win32 Then
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnMTOptions", True, mtbIDMTOptions
#Else
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnMTOptions", True, mtbIDMTOptionsMac
#End If
End Sub

'Set Browse Type
Public Sub MTCommand_OnBrowseType(control As IRibbonControl, selectedId As String, selectedIndex As Integer)

    'RunMTDLLCommand moduleCLS, "NoDirectCall_OnBrowseType"
    ' We can't use RunMTDLLCommand since we need a way to pass arguments.
    ' For now, I just instantiate a guard directly and did not
    ' provide an error handler in light of the simplicity of the code
    ' to be executed.

    Dim toplevelguard As CommandGuard
    Set toplevelguard = New CommandGuard

    If Application.name = kAppMSW Then
        ' first index comes through as 0, so add one
        gMTBrowseType = selectedIndex + 1
    End If

End Sub

'Browse to Previous
Public Sub MTCommand_OnBrowsePrevious(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnBrowsePrevious", True, mtbIDBrowsePrev
End Sub

'Browse to Next
Public Sub MTCommand_OnBrowseNext(control As IRibbonControl)
    RunMTDLLCommand moduleCLS, "NoDirectCall_OnBrowseNext", True, mtbIDBrowseNext
End Sub

Public Sub MTCallback_GetEnabled(control As IRibbonControl, ByRef enabled)

#If MAC_OFFICE_VERSION >= 15 Then
    If Val(Application.version) < 15.38 Then
        If Not UserInformed Then
           UserInformed = True
           MsgBox ("MathType works with Microsoft Word 15.38 or higher. Please update your version of Word")
        End If
        enabled = False 'if version is less than 15.38 all buttons are disbaled
    Else
        If control.id = mtbIDMathPage Then
            enabled = False 'Publish to mathpage doesn't work for word2016mac, hence disabled
        Else
            enabled = RunUICallback("NoDirectCall_GenEnabledByAppFunctionality", control.id)
        End If
    End If
#Else
        enabled = RunUICallback("NoDirectCall_GenEnabledByAppFunctionality", control.id)
#End If

    End Sub

Public Sub MTCallback_GetSupertip(control As IRibbonControl, ByRef screentip)
    screentip = RunUICallback("NoDirectCall_LocateSupertip", control.id)
End Sub

Public Sub MTCallback_GetLabel(control As IRibbonControl, ByRef label)
    label = RunUICallback("NoDirectCall_GetUnlockUI")
End Sub

Public Sub MTCallback_GetScreenTip(control As IRibbonControl, ByRef screentip)
    screentip = RunUICallback("NoDirectCall_GetUnlockScreenTipUI")
End Sub

Public Sub MTCallback_GetNumEqVisible(control As IRibbonControl, ByRef visible)
    visible = RunUICallback("NoDirectCall_GetNumEqVisible", control.id)
End Sub

Public Sub MTCallback_IsWin7(control As IRibbonControl, ByRef visible)
    #If Win32 Then
        visible = IsMathInputPanelAvailable
    #Else
        visible = False
    #End If
End Sub

Public Sub MTCallback_IsBeforeWin7(control As IRibbonControl, ByRef visible)
    #If Win32 Then
        visible = Not IsMathInputPanelAvailable
    #Else
        visible = False
    #End If
End Sub

Public Sub MTCallback_IsWin(control As IRibbonControl, ByRef visible)
    #If Win32 Then
        visible = True
    #Else
        visible = False
    #End If
End Sub

Public Sub MTCallback_IsMac(control As IRibbonControl, ByRef visible)
    #If Win32 Then
        visible = False
    #Else
        visible = True
    #End If
End Sub


