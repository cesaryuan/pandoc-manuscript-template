Attribute VB_Name = "UILibCls"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False

'=====================================================================
' (c) Copyright 2007-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UILibCls.cls 7     10/11/11 2:12p Jimm $
'=====================================================================

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

Public Sub NoDirectCall_Browse()
    Dim userChoice As Integer
    Dim dropDown As Office.CommandBarComboBox
    Dim command As String

    If browseChoice = -1 Then
        ' serach for the only combo box
        Set dropDown = CommandBars("MathType").FindControl(msoControlDropdown)
        If Not dropDown Is Nothing Then
            userChoice = dropDown.ListIndex
        Else
            userChoice = 1
        End If
    Else
        userChoice = browseChoice
    End If

    If forward Then
        Select Case userChoice
        Case 0 To 1
            command = kMTCommands & ".MTBrowse.BrowseEquationsForward"
        Case 2
            command = kMTCommands & ".MTBrowse.BrowseEquationNumberRefForward"
        Case 3
            command = kMTCommands & ".MTBrowse.BrowseChapterSectionForward"
        End Select
    Else
        Select Case userChoice
        Case 0 To 1
            command = kMTCommands & ".MTBrowse.BrowseEquationsBackward"
        Case 2
            command = kMTCommands & ".MTBrowse.BrowseEquationNumberRefBackward"
        Case 3
            command = kMTCommands & ".MTBrowse.BrowseChapterSectionBackward"
        End Select
    End If

    If Not command = "" Then
        RunDispatch "", command, False
    End If
    MTIncrementStatisticBy "WDBr", 1
End Sub

Public Sub NoDirectCall_InsertInlineEqn()
    WriteLog "Entering InsertInlineEqn"
    RunDispatch "", kMTCommands & ".MTInsertEquation.InsertInlineEquation", False
    
    WriteLog "Incrementing CInline VC stat"
    MTIncrementStatisticBy "CInline", 1
    
    WriteLog "Exiting InsertInlineEqn"
End Sub

Public Sub NoDirectCall_InsertDispEqn()
    RunDispatch "", kMTCommands & ".MTInsertEquation.InsertDisplayEquation", False
    MTIncrementStatisticBy "CDisp", 1
End Sub

Public Sub NoDirectCall_InsertLeftNumberedDispEqn()
    RunDispatch "", kMTCommands & ".MTInsertEquation.InsertLeftNumberedDisplayEquation", False
    MTIncrementStatisticBy "CDispL", 1
End Sub

Public Sub NoDirectCall_InsertRightNumberedDispEqn()
    RunDispatch "", kMTCommands & ".MTInsertEquation.InsertRightNumberedDisplayEquation", False
    MTIncrementStatisticBy "CDispR", 1
End Sub

Public Sub NoDirectCall_InsertLeftNumberedDispEqnEB()
    RunDispatch "", kMTCommands & ".MTInsertEBEquation.InsertEBLeftNumberedDisplayEquation", False
    MTIncrementStatisticBy "CEBDispL", 1
End Sub

Public Sub NoDirectCall_InsertRightNumberedDispEqnEB()
    RunDispatch "", kMTCommands & ".MTInsertEBEquation.InsertEBRightNumberedDisplayEquation", False
    MTIncrementStatisticBy "CEBDispR", 1
End Sub

Public Sub NoDirectCall_InsertEqnNum()
    RunDispatch "", kMTCommands & ".MTEqnNum.DlgMain", False
    MTIncrementStatisticBy "CEqNum", 1
End Sub

Public Sub NoDirectCall_InsertEqnRef()
    RunDispatch "", kMTCommands & ".MTMarkRef.DlgMain", False
    MTIncrementStatisticBy "CEqRef", 1
End Sub

Public Sub NoDirectCall_UpdateEqns()
    RunDispatch "", kMTCommands & ".MTUpdateFields.DlgMain", False
    MTIncrementStatisticBy "CUpdEqNum", 1
End Sub

Public Sub NoDirectCall_CrashTest()
    Dim result As Long
    Dim where, how
    where = 1 '1 for WLL; 2 for DLL
    how = 1   '1 for Assert; 2 for divide by zero
    result = MTCrashTest(where, how)
End Sub

'checks if Word is being used as an embedded object
'if Word's window does not have a caption, we assume it is embedded
Private Function IsEmbeddedObject() As Boolean
    IsEmbeddedObject = (Len(Application.ActiveWindow.caption) = 0)
End Function
