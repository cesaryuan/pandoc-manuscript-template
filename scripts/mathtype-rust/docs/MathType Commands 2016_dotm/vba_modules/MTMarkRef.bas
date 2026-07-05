Attribute VB_Name = "MTMarkRef"
'MTMarkRef 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTMarkRef.bas 26    10/11/11 2:12p Jimm $
'=====================================================================
'Handles the Insert menu's Equation Reference command.
'Places a bookmark "Reference" at the current location,
'for use with the "MTPlaceRef" macro

Public gDlgCanceled As Boolean

Option Explicit

Public Sub DlgMain()
Attribute DlgMain.VB_Description = "Places, at the current insertion point, a reference to a user-selectable equation number."
Attribute DlgMain.VB_ProcData.VB_Invoke_Func = "TemplateProject.MTMarkRef.MAIN"
    Dim dontShow As String
    Dim title As String

    title = MTLib.GetUserString("!1901Insert Equation Reference")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo abort
    End If

    'if there is a selection, collapse to start
    If Selection.Type <> wdSelectionIP Then
        Selection.Collapse direction:=wdCollapseStart
    End If

    With Selection
        .InsertAfter MTLib.GetUserString("!1903equation reference goes here")
                #If Win32 Then
        .ItalicRun
                #End If
    End With
    Application.ScreenRefresh

    'place the bookmark and let the user know we're ready
    On Error Resume Next
    ActiveDocument.Bookmarks.Add name:="MTReference", Range:=Selection.Range
    dontShow = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNREFDLG)
    If dontShow <> "1" Then
        MTInsertEqnRefDlg.Show
        If gDlgCanceled Then
            ActiveDocument.Bookmarks("MTReference").delete
            With Selection
                                #If Win32 Then
                    .ItalicRun
                                #End If
                    .delete
            End With
            Application.ScreenRefresh
            GoTo abort
        End If
    End If

    'write property containing active pane ID for MTPlaceRef to use
    If ActiveWindow.Panes.count > 1 Then
        Dim paneID
        paneID = ActiveWindow.ActivePane.index
        MTLib.WriteDocPropString$ ActiveDocument, mtprop_EQNREFPANE, Strings.LTrim$(Conversion.str$(paneID))
    Else
        MTLib.DeleteDocProperty ActiveDocument, mtprop_EQNREFPANE
    End If

    'display statusbar message anyway
    Application.StatusBar = MTLib.GetUserString("!1902Double-click on the equation number you want to reference")

abort:
End Sub

