Attribute VB_Name = "MTSetData"

'MTLib: MTSetData
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTSetData.bas 9     2/10/14 1:49p Jimm $
'=====================================================================

Option Explicit

'return True on success, False on error
Public Function SetMTData(eqnShape As InlineShape, mmlStr As String) As Boolean

    Dim isError As Boolean
    'assume error
    isError = True

    On Error GoTo shutdown

    Dim myObj As Object
    Dim stat As Long

    MTLib.WriteLog "Entering SetMTData"

    ActivateMT eqnShape
    Set myObj = eqnShape.OLEFormat.Object
    
    Dim mmlUnicode() As Byte
    mmlUnicode = mmlStr
    stat = MTSetEqnFromLangStr(myObj, mtlangMATHML, mmlUnicode(0), Len(mmlStr))
    If (stat = mtOK) Then
        MTLib.WriteLog "MTSetEqnFromLangStr called successfully"
        isError = False
        MTLib.WriteLog "SetData called successfully"
    Else
        MTLib.WriteLog "Error in MTSetEqnFromLangStr: " & stat
        isError = True
    End If

shutdown:
    stat = ShutdownMT(eqnShape)

    If stat <> mtOK Or isError Then
        Dim userAction
        userAction = ShowTransformError(stat, MTLib.GetUserString2("1658", "3310", "MathType Automation Error Setting or Closing Object"))
    End If
    MTLib.WriteLog "Exiting SetMTData"
    SetMTData = Not isError

End Function

Sub ActivateMT(eqnShape As InlineShape)

    ' Enhancement? look up RunForConversion verb
    eqnShape.OLEFormat.DoVerb (2)
    MTLib.WriteLog "MT Equation Activated"

End Sub

Function ShutdownMT(eqnShape As InlineShape) As Long

    Dim closeObj As Object
    Dim stat As Long

    eqnShape.Range.Fields.update 'In Word 2003, ActiveDocument.Fields.update skips text boxes...
    DoEvents

    Set closeObj = eqnShape.OLEFormat.Object
    stat = MTCloseOleObject(1, closeObj)
    If stat = mtOK Then
        MTLib.WriteLog "MT eqn object closed"
    Else
        MTLib.WriteLog "Couldn't call close on MT eqn object"
    End If

    ' Grisly hack for the Word 2003 corruption bug
    ' assumes typing replaces selection is off.  Tex Toggle and
    ' Convert Equations set it that way before using IDataObject.
    If Val(Application.version) = kWord2003 Then
        Dim tmp As Range
        Dim tmp2 As Range
        Set tmp = Selection.Range

        eqnShape.Select
        Selection.Copy
        Selection.Paste ' forces the equation to be recreated
        'Word 2003 ignores the "typing replaces text" setting
        'for pastes within a table.  Joy.  MT-2100
        If Not tmp.Information(wdWithInTable) Then
            On Error Resume Next
            eqnShape.Range.delete
        End If

        Set tmp2 = Selection.Range
        tmp2.SetRange tmp2.start - 27, tmp2.end
        If tmp2.InlineShapes.count > 0 Then
            Set eqnShape = tmp2.InlineShapes(1)
        Else
            Set eqnShape = Nothing
        End If

        tmp.Select
    End If

    ShutdownMT = stat

End Function
