Attribute VB_Name = "UIWrappers2007"
'UIWrappers2007: 5.01
'====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved
'$Header: /MathType/Windows/WordMacros/UIWrappers2007.bas 2     10/11/11 2:12p Jimm $
'====================================================================
Option Explicit

Private Const module As String = "UIWrappers"
Private Const moduleCLS As String = "UIWrappersCls"
Public currentClipboardFormat As ClipboardFormat

Public Sub DSIEditPaste(control As IRibbonControl, cancelDefault)
    If InitEditPaste <> True Then
        cancelDefault = False
    End If
End Sub

#If MAC_OFFICE_VERSION >= 15 Then
'the "idMso=paste" in ribbon doesn't capture all the copy actions, therefore, adding following
Public Sub EditPaste()
     If InitEditPaste <> True Then
        Selection.Paste
    End If
End Sub
#End If

Private Function InitEditPaste() As Boolean

     If IsMTEquationOnClipboard = True Then
        Exit Function
    End If
    
     ' Since translators are off in Lite Mode, there's no point in checking.
    If (MTIsFullFunctionality() = 1) Then
    
        'If MathML is on the clipboard in CF_TEXT format, paste it
        currentClipboardFormat = ClipboardFormat.kText
        
        Dim stat As Long
        
        If (IsOnlyMathMLOnClipboard = True) Then
            stat = mtOK
        Else
            stat = mtNOT_EQUATION
        End If
        
        If (stat <> mtOK) Then
            'If MathML is on the clipboard and CF_TEXT is not on the clipboard, paste it
            'the Math Input Panel only copies the MathML flavor to the clipboard
            currentClipboardFormat = ClipboardFormat.kMathML
            If MTMacrosInitialized Then
            #If Win32 Then
                Dim isMML As Boolean
                MTIsClipboardFormatAvailable "MathML", 0, isMML
                Dim isCFText As Boolean
                MTIsClipboardFormatAvailable "", CF_TEXT, isCFText
                    If isMML And Not isCFText Then
                        stat = mtOK
                    End If
            #End If
            End If
        End If
        
        If (stat = mtOK) Then
            RunMTDLLCommand moduleCLS, "NoDirectCall_EditPaste"
            InitEditPaste = True
            Exit Function
        End If
        
    End If
End Function
Public Function IsMTEquationOnClipboard() As Boolean
    On Error Resume Next
    #If Win32 Then
        Dim epsDlg As Dialog
        Set epsDlg = Dialogs(wdDialogEditPasteSpecial)
        If err.Number = 0 Then
            With epsDlg
                If .Class = "Equation" Or Strings.left$(.Class, 9) = "Equation." Or Strings.left$(.caption, 8) = "MathType" Then
                    .floating = False
                    IsMTEquationOnClipboard = True
                End If
            End With
        End If
    #Else
        With Dialogs(wdDialogEditPasteSpecial)
            If .Class = "Equation" Or Strings.left$(.Class, 9) = "Equation." Then
                .floating = False
                IsMTEquationOnClipboard = True
            End If
        End With
    #End If
End Function
