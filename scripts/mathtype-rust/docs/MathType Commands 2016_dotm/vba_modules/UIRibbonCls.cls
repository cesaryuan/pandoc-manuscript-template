Attribute VB_Name = "UIRibbonCls"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False

'=====================================================================
' (c) Copyright 2007-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UIRibbonCls.cls 18    6/23/10 6:50p Robertm $
'=====================================================================

Private Const kUILib As String = "UILibCls"
Private Const kPPInsertMTEquation As String = "PPInsertMTEquationCls"
Private Const kUIHelp As String = "UIHelpCls"
Private Const kUIBrowse As String = "PPBrowseCls"

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

Public Sub NoDirectCall_OnRibbonLoaded()
  Set gMTRibbon = ribbon
End Sub

Public Sub NoDirectCall_OnSelectedIndexBrowseType()
    If Application.name = kAppMSW Then
        If gMTBrowseType = 0 Then
            index = gMTBrowseType
        Else
            ' adjust to zero based index before setting the value
            index = gMTBrowseType - 1
        End If
    End If
End Sub

Public Sub NoDirectCall_OnInsertInlineEqn()
    If Application.name = kAppMSW Then
        RunDispatch kUILib, "NoDirectCall_InsertInlineEqn"
    ElseIf Application.name = kAppMSPP Then
        RunDispatch kPPInsertMTEquation, "NoDirectCall_OnInsertEquation"
    End If
End Sub

Public Sub NoDirectCall_OnInsertDispEqn()
    If Application.name = kAppMSW Then
        RunDispatch kUILib, "NoDirectCall_InsertDispEqn"
    End If
End Sub

Public Sub NoDirectCall_OnInsertLeftNumberedDispEqn()
    If Application.name = kAppMSW Then
        #If Word Then
        If gMTEqnNumsOnRight = True Then
            gMTEqnNumsOnRight = False
            #If Win32 Or MAC_OFFICE_VERSION < 15 Then
            Invalidate
            #End If
            SaveDefaultEqnNumSide
        End If
        RunDispatch kUILib, "NoDirectCall_InsertLeftNumberedDispEqn"
        #End If
    End If
End Sub

Public Sub NoDirectCall_OnInsertRightNumberedDispEqn()
    If Application.name = kAppMSW Then
        #If Word Then
        If gMTEqnNumsOnRight = False Then
            gMTEqnNumsOnRight = True
            #If Win32 Or MAC_OFFICE_VERSION < 15 Then
            Invalidate
            #End If
            SaveDefaultEqnNumSide
        End If
        RunDispatch kUILib, "NoDirectCall_InsertRightNumberedDispEqn"
        #End If
    End If
End Sub

Public Sub NoDirectCall_OnInsHandEqn()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTMathInputControl.DlgMain", False, mtbIDInsHandEqn
    ElseIf Application.name = kAppMSPP Then
        RunDispatch kPPInsertMTEquation, "NoDirectCall_OnInsPPHandEqn"
    End If
End Sub

Public Sub NoDirectCall_OnInsertInlineEqnEB()
    #If Word Then
        If Documents.count > 0 Then
            cancelDefault = False
            MTIncrementStatisticBy "CEBInline", 1
        End If
    #ElseIf PP Then
        If Presentations.count > 0 Then
            cancelDefault = False
            MTIncrementStatisticBy "CEBInline", 1
        End If
    #End If
End Sub

Public Sub NoDirectCall_OnInsertLeftNumberedDispEqnEB()
    If Application.name = kAppMSW Then
        RunDispatch kUILib, "NoDirectCall_InsertLeftNumberedDispEqnEB"
    End If
End Sub

Public Sub NoDirectCall_OnInsertRightNumberedDispEqnEB()
   If Application.name = kAppMSW Then
        RunDispatch kUILib, "NoDirectCall_InsertRightNumberedDispEqnEB"
   End If
End Sub


Public Sub NoDirectCall_OnFormatEquations()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTFormatEquations.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnConvertEquations()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTConvertEquations.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnTeXToggle()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTTeXToggle.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnExportEquations()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTExportEquations.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnMathPage()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MathPage.MP_ExportTo", False
    End If
End Sub

Public Sub NoDirectCall_OnSetEquationPrefs()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTSetEqnPrefs.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnInsertEquationNumber()
    If Application.name = kAppMSW Then
        RunDispatch kUILib, "NoDirectCall_InsertEqnNum"
    End If
End Sub


Public Sub NoDirectCall_OnFormatEquationNumbers()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTEqnNumFormat.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnUpdateEquationNumbers()
    If Application.name = kAppMSW Then
        RunDispatch kUILib, "NoDirectCall_UpdateEqns", True
    End If
End Sub

Public Sub NoDirectCall_OnEquationReference()
    If Application.name = kAppMSW Then
        RunDispatch kUILib, "NoDirectCall_InsertEqnRef"
    End If
End Sub


Public Sub NoDirectCall_OnInsertChapterSectionBreak()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTSecNum.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnInsertNextSection()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTSecNum.InsertNextSectionBreak", False
    End If
End Sub

Public Sub NoDirectCall_OnInsertNextChapter()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTSecNum.InsertNextChapterBreak", False
    End If
End Sub

Public Sub NoDirectCall_OnModifyChapterSectionBreak()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTFormatEquationSection.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnHelpContents()
    RunDispatch kUIHelp, "NoDirectCall_ShowHelpContents"
End Sub

Public Sub NoDirectCall_OnHelpMTInWord()
     RunDispatch kUIHelp, "NoDirectCall_ShowHelpMTInWord"
End Sub

Public Sub NoDirectCall_OnUnlockReg()
    RunDispatch kUIHelp, "NoDirectCall_ShowUnlockReg"
End Sub

Public Sub NoDirectCall_OnAboutMT()
    RunDispatch kUIHelp, "NoDirectCall_ShowAboutMT"
End Sub

Public Sub NoDirectCall_OnWebHomePage()
    RunDispatch kUIHelp, "NoDirectCall_ShowWebHomePage"
End Sub

Public Sub NoDirectCall_OnWebSupport()
    RunDispatch kUIHelp, "NoDirectCall_ShowWebSupport"
End Sub

Public Sub NoDirectCall_OnWebEmailFeedback()
    RunDispatch kUIHelp, "NoDirectCall_ShowWebEmailFeedback"
End Sub

Public Sub NoDirectCall_OnWebOrderMathType()
    RunDispatch kUIHelp, "NoDirectCall_ShowWebOrderMathType"
End Sub

Public Sub NoDirectCall_OnFutureMT()
    RunDispatch kUIHelp, "NoDirectCall_ShowFutureMT"
End Sub

Public Sub NoDirectCall_OnMTOptions()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTOptions.DlgMain", False
    End If
End Sub

Public Sub NoDirectCall_OnBrowseType()
    If Application.name = kAppMSW Then
        ' first index comes through as 0, so add one
        gMTBrowseType = selectedIndex + 1
    End If
End Sub

Public Sub NoDirectCall_OnBrowsePrevious()
    If Application.name = kAppMSW Then
        browseChoice = gMTBrowseType
        forward = False
        RunDispatch kUILib, "NoDirectCall_Browse"
    ElseIf Application.name = kAppMSPP Then
        forward = False
        RunDispatch kUIBrowse, "NoDirectCall_Browse"
    End If
End Sub

Public Sub NoDirectCall_OnBrowseNext()
    If Application.name = kAppMSW Then
        browseChoice = gMTBrowseType
        forward = True
        RunDispatch kUILib, "NoDirectCall_Browse"
    ElseIf Application.name = kAppMSPP Then
        forward = True
        RunDispatch kUIBrowse, "NoDirectCall_Browse"
    End If
End Sub

Public Sub NoDirectCall_OnMathInputControl()
    If Application.name = kAppMSW Then
        RunDispatch "", kMTCommands & ".MTMathInputControl.DlgMain", False
    End If
End Sub
