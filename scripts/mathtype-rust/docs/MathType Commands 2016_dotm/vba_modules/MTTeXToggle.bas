Attribute VB_Name = "MTTeXToggle"
'MTTeXToggle
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'=====================================================================
Option Explicit

Private Const loopCutoff = 500 'max number equations to process, etc.

Public gMTConvertToInlineResult As String
Public gMTConvertToDisplayResult As String

Public Sub DlgMain()
    Dim rng As Range, tmpRng As Range
    Dim wordState As Long
    Set rng = Selection.Range

    On Error GoTo cleanup

    'save SmartCut&Paste setting(s) & TypingReplacesSelection settings, & turn them off
    wordState = MTLib.SaveWordState
    options.SmartCutPaste = False
    If Val(Application.version) >= kWordX Then
       MTLib.SetPasteSmartCutPaste False
    End If
    options.ReplaceSelection = False
    ActiveDocument.TrackRevisions = False

    'be sure the bookmarks are clean before starting
    If ActiveDocument.Bookmarks.Exists("MTToggleStart") Then
        ActiveDocument.Bookmarks("MTToggleStart").delete
    End If
    If ActiveDocument.Bookmarks.Exists("MTToggleEnd") Then
        ActiveDocument.Bookmarks("MTToggleEnd").delete
    End If

    'if rng ends with a cell marker, trim it off, and yes end -1 is correct -- look it up
     If Strings.right(rng.Text, 2) = Strings.Chr(13) & Strings.Chr(7) Then rng.SetRange rng.start, rng.end - 1

    'bookmark the original selection
    Set tmpRng = CopyRange(rng)
    tmpRng.Collapse wdCollapseStart
    ActiveDocument.Bookmarks.Add name:="MTToggleStart", Range:=tmpRng
    Set tmpRng = CopyRange(rng)
    tmpRng.Collapse wdCollapseEnd
    ActiveDocument.Bookmarks.Add name:="MTToggleEnd", Range:=tmpRng

    Dim curPos As Long
    Dim eqnRng As Range
    curPos = rng.start
    MTLib.SetScreenUpdate False
    If rng.start < rng.end Then 'toggle equations within a Selection
        Dim cutoff As Integer

        'Add a period, to keep the ends separated, in case the selection
        'is exactly one equation
        'The table hack is because modifying rng changes Selection as a
        'side effect if rng intersects a table
        Dim tableHack As Range
        Set tableHack = Selection.Range
        rng.InsertAfter "."
        tableHack.Select

        'shift bookmark to include period
        ActiveDocument.Bookmarks("MTToggleEnd").end = ActiveDocument.Bookmarks("MTToggleEnd").end + 1
        ActiveDocument.Bookmarks("MTToggleEnd").start = ActiveDocument.Bookmarks("MTToggleEnd").start + 1

        Do
            cutoff = cutoff + 1
            Set eqnRng = FindNextEquation(rng, "forward", curPos)
            If Not eqnRng Is Nothing Then
                toggleEqn eqnRng
                curPos = eqnRng.end
            End If
        Loop Until eqnRng Is Nothing Or cutoff > loopCutoff

        If ActiveDocument.Bookmarks.Exists("MTToggleEnd") Then
            'move to end bookmark, delete period
            tmpRng.SetRange ActiveDocument.Bookmarks("MTToggleEnd").start - 1, _
                ActiveDocument.Bookmarks("MTToggleEnd").end
            ' just being defensive...
            If tmpRng.Text = "." Then
            tmpRng.delete
            End If
        End If

    Else 'search for nearest equation to insertion point
        Dim success As Boolean

        If Selection.Information(wdWithInTable) Then
            rng.Expand wdCell
            rng.SetRange rng.start, rng.end - 1
        Else
            rng.Expand (wdStory)
        End If

        success = ToggleNearestEquationInRange(rng, curPos)
    End If

    'restore original range and clean up
    If ActiveDocument.Bookmarks.Exists("MTToggleStart") Then
        If ActiveDocument.Bookmarks.Exists("MTToggleEnd") Then
            tmpRng.SetRange ActiveDocument.Bookmarks("MTToggleStart").start, _
                ActiveDocument.Bookmarks("MTToggleEnd").end
            tmpRng.Select 'this doesn't work in all situations.  See MT-17XX
            ActiveDocument.Bookmarks("MTToggleEnd").delete
        End If
            ActiveDocument.Bookmarks("MTToggleStart").delete
        End If

cleanup:
    MTIncrementStatisticBy "TeXToggle", 1
    MTLib.RestoreWordState (wordState)
    MTLib.SetScreenUpdate True

End Sub

Public Sub DoTexToggle(rngType As Long, count As Long)
    Dim rng As Range, tmpRng As Range
    Dim wordState As Long

    If rngType = 1 Then 'toggle equations within the current selection
        Set rng = Selection.Range
    ElseIf rngType = 2 Then 'toggle equations within the whole document
        Set rng = ActiveDocument.Range
    End If

    On Error GoTo cleanup

    'save SmartCut&Paste setting(s) & TypingReplacesSelection settings, & turn them off
    wordState = MTLib.SaveWordState
    options.SmartCutPaste = False
    If Val(Application.version) >= kWordX Then
       MTLib.SetPasteSmartCutPaste False
    End If
    options.ReplaceSelection = False
    ActiveDocument.TrackRevisions = False

    'be sure the bookmarks are clean before starting
    If ActiveDocument.Bookmarks.Exists("MTToggleStart") Then
        ActiveDocument.Bookmarks("MTToggleStart").delete
    End If
    If ActiveDocument.Bookmarks.Exists("MTToggleEnd") Then
        ActiveDocument.Bookmarks("MTToggleEnd").delete
    End If

    'if rng ends with a cell marker, trim it off, and yes end -1 is correct -- look it up
    If Strings.right(rng.Text, 2) = Strings.Chr(13) & Strings.Chr(7) Then rng.SetRange rng.start, rng.end - 1

    'bookmark the original selection
    Set tmpRng = CopyRange(rng)
    tmpRng.Collapse wdCollapseStart
    ActiveDocument.Bookmarks.Add name:="MTToggleStart", Range:=tmpRng
    Set tmpRng = CopyRange(rng)
    tmpRng.Collapse wdCollapseEnd
    ActiveDocument.Bookmarks.Add name:="MTToggleEnd", Range:=tmpRng

    Dim curPos As Long
    Dim eqnRng As Range
    curPos = rng.start
    MTLib.SetScreenUpdate False

    If rngType = 0 Then 'search for nearest equation to insertion point
        Dim success As Boolean

        If Selection.Information(wdWithInTable) Then
            rng.Expand wdCell
            rng.SetRange rng.start, rng.end - 1
        Else
            rng.Expand (wdStory)
        End If

        success = ToggleNearestEquationInRange(rng, curPos, False)

        If success Then
            count = count + 1
        End If

    ElseIf rng.start < rng.end Then
        Dim cutoff As Integer

        'Add a period, to keep the ends separated, in case the selection
        'is exactly one equation
        'The table hack is because modifying rng changes Selection as a
        'side effect if rng intersects a table
        Dim tableHack As Range
        Set tableHack = Selection.Range
        rng.InsertAfter "."
        tableHack.Select

        'shift bookmark to include period
        ActiveDocument.Bookmarks("MTToggleEnd").end = ActiveDocument.Bookmarks("MTToggleEnd").end + 1
        ActiveDocument.Bookmarks("MTToggleEnd").start = ActiveDocument.Bookmarks("MTToggleEnd").start + 1

        Do
            cutoff = cutoff + 1
            Set eqnRng = FindNextEquation(rng, "forward", curPos)
            If Not eqnRng Is Nothing Then
                toggleEqn eqnRng, False
                curPos = eqnRng.end
                count = count + 1
            End If
        Loop Until eqnRng Is Nothing Or cutoff > loopCutoff

        If ActiveDocument.Bookmarks.Exists("MTToggleEnd") Then
            'move to end bookmark, delete period
            tmpRng.SetRange ActiveDocument.Bookmarks("MTToggleEnd").start - 1, _
                ActiveDocument.Bookmarks("MTToggleEnd").end
            ' just being defensive...
            If tmpRng.Text = "." Then
            tmpRng.delete
            End If
        End If
    End If

    'restore original range and clean up
    If ActiveDocument.Bookmarks.Exists("MTToggleStart") Then
        If ActiveDocument.Bookmarks.Exists("MTToggleEnd") Then
            tmpRng.SetRange ActiveDocument.Bookmarks("MTToggleStart").start, _
                ActiveDocument.Bookmarks("MTToggleEnd").end
            tmpRng.Select 'this doesn't work in all situations.  See MT-17XX
            ActiveDocument.Bookmarks("MTToggleEnd").delete
        End If
            ActiveDocument.Bookmarks("MTToggleStart").delete
        End If

cleanup:
    MTIncrementStatisticBy "TeXToggle", 1
    MTLib.RestoreWordState (wordState)
    MTLib.SetScreenUpdate True

End Sub

Function ToggleNearestEquationInRange(rng As Range, curPos As Long, Optional ShowDialog As Boolean = True) As Boolean
    Dim eqnRng As Range
    ToggleNearestEquationInRange = True
        Set eqnRng = InEqn(rng, curPos)
        If Not eqnRng Is Nothing Then
            toggleEqn eqnRng, ShowDialog
        Exit Function
        End If

        Dim fEqn As Range
        Dim bEqn As Range
        Set fEqn = FindNextEquation(rng, "forward", curPos)
        Set bEqn = FindNextEquation(rng, "backward", curPos)

        ' Now we check to see if the cursor is online with either equation
        Dim tmpRng As Range
        Dim lineBegin As Integer, lineEnd As Integer, lineCursor As Integer
        Dim fCursor As Boolean, bCursor As Boolean
        fCursor = False
        bCursor = False

        lineCursor = Selection.Information(wdFirstCharacterLineNumber)
        If (lineCursor = -1) Then
            lineCursor = Selection.Range.Information(wdFirstCharacterLineNumber)
        End If

        If Not fEqn Is Nothing Then
            Set tmpRng = CopyRange(fEqn)
            lineBegin = tmpRng.Information(wdFirstCharacterLineNumber)
            tmpRng.Collapse wdCollapseEnd
            lineEnd = tmpRng.Information(wdFirstCharacterLineNumber)
            ' the case where the cursor is in the middle of a multiline
            ' equation is handled above by inEqn, and using <= and =>
            ' screws up over page breaks
            If lineBegin = lineCursor Or lineEnd = lineCursor Then
                fCursor = True
            End If
        End If
        If Not bEqn Is Nothing Then
            Set tmpRng = CopyRange(bEqn)
            lineBegin = tmpRng.Information(wdFirstCharacterLineNumber)
            tmpRng.Collapse wdCollapseEnd
            lineEnd = tmpRng.Information(wdFirstCharacterLineNumber)
            If lineBegin = lineCursor Or lineEnd = lineCursor Then
                bCursor = True
            End If
        End If


        If fCursor Then
            If Not bCursor Then
                toggleEqn fEqn, ShowDialog
            Exit Function
            Else
                If Abs(curPos - fEqn.start) < Abs(curPos - bEqn.end) Then
                    toggleEqn fEqn, ShowDialog
                Exit Function
                Else
                    toggleEqn bEqn, ShowDialog
                Exit Function
                End If
            End If
        Else
            If bCursor Then
                toggleEqn bEqn, ShowDialog
                Exit Function
            End If
        End If
    ToggleNearestEquationInRange = False
End Function

' returns a Range containing the equation if found, Null otherwise
Function FindNextEquation(rng As Range, dir As String, ByVal pos As Long) As Range
    Dim iEqn As Range, dEqn As Range, mtEqn As Range

    Set iEqn = FindNextInlineTeXEqn(rng, dir, pos)
    Set dEqn = FindNextDisplayTeXEqn(rng, dir, pos)
    Set mtEqn = FindNextMTEqn(rng, dir, pos)

    Dim returnVal As Range
    Set returnVal = Nothing
    If Not iEqn Is Nothing Then
        Set returnVal = iEqn
    End If
    If Not dEqn Is Nothing Then
        If returnVal Is Nothing Then
            Set returnVal = dEqn
        Else
            Set returnVal = closerEqn(returnVal, dEqn, dir)
        End If
    End If
    If Not mtEqn Is Nothing Then
        If returnVal Is Nothing Then
            Set returnVal = mtEqn
        Else
            Set returnVal = closerEqn(returnVal, mtEqn, dir)
        End If
    End If
    Set FindNextEquation = returnVal
End Function

' returns a Range containing the equation if found, Nothing otherwise
Function FindNextInlineTeXEqn(rng As Range, dir As String, ByVal pos As Long) As Range
    Dim delim1 As Long, delim2 As Long
    Dim offset As Long
    Dim returnRng As Range
    Dim cutoff As Integer

    Set returnRng = CopyRange(rng)

    Do
        cutoff = cutoff + 1
        delim1 = search(rng, dir, pos, "$")
        If delim1 = -1 Then
            ' no more candidate start delims to find, so return
            Set FindNextInlineTeXEqn = Nothing
            Exit Function
        End If

        If dir = "forward" Then
            delim2 = search(rng, dir, delim1 + 1, "$")
        Else
            delim2 = search(rng, dir, delim1, "$")
        End If

        If delim2 = -1 Then
            If dir = "forward" Then
                Set FindNextInlineTeXEqn = Nothing
                Exit Function
            Else
                ' when we are searching backwards, this may not signal we
                ' are done, since there may be an unmatched trailing delimiter
                ' e.g. in a dollar amount, etc. So merely ensure returnRng isn't valid TeX
                ' so the loop continues
                returnRng.SetRange 1, 1
            End If
        Else
            If dir = "forward" Then
                returnRng.SetRange delim1, delim2 + 1
            Else
                returnRng.SetRange delim2, delim1 + 1
            End If
        End If

        If IsValidTeX(returnRng) Then
            Set FindNextInlineTeXEqn = returnRng
            Exit Function
        ElseIf dir = "forward" Then
            pos = delim2
        Else
            pos = delim1
        End If
    Loop While True And cutoff < loopCutoff
End Function

' returns a Range containing the equation if found, Nothing otherwise
Function FindNextDisplayTeXEqn(rng As Range, dir As String, ByVal pos As Long) As Range
    Dim startPos As Long, endPos As Long
    Dim offset As Long
    Dim returnRng As Range
    Set returnRng = CopyRange(rng)

    Dim cutoff As Integer
    Do
        cutoff = cutoff + 1
        startPos = search(rng, dir, pos, Strings.ChrW(&H5C) + "[")
        If startPos = -1 Then
            ' no more candidate start delims to find, so return
            Set FindNextDisplayTeXEqn = Nothing
            Exit Function
        End If

        endPos = search(rng, "forward", startPos + 2, Strings.ChrW(&H5C) + "]")
        If endPos = -1 Then
            ' could have a syntax error, so merely ensure returnRng isn't valid TeX
            returnRng.SetRange 1, 1
        Else
            returnRng.SetRange startPos, endPos + 2
        End If

        If IsValidTeX(returnRng) Then
            Set FindNextDisplayTeXEqn = returnRng
            Exit Function
        Else
            ' we want to search from just past the start delimiter, not the end delimiter
            If dir = "forward" Then
                pos = startPos + 2
            Else
                pos = startPos
            End If
        End If

    Loop While True And cutoff < loopCutoff

End Function

' returns a Range containing the equation if found, Nothing otherwise
Function FindNextMTEqn(rng As Range, dir As String, ByVal pos As Long) As Range
    Dim myRng As Range
    Dim shp As InlineShape
    Dim i As Integer, eqnType As Integer, eqnIdx As Integer, closest As Integer

    Set myRng = rng
    eqnIdx = -1
    closest = rng.end - rng.start + 1 'larger than the distance to any point in rng

    If myRng.InlineShapes.count <= 0 Then
        Set FindNextMTEqn = Nothing
        Exit Function
    Else
        For i = 1 To myRng.InlineShapes.count
            Set shp = myRng.InlineShapes(i)
            eqnType = 0
            If shp.Type = wdInlineShapeEmbeddedOLEObject Then
                'accessing ProgID may cause an error 5825 on corrupt objects, so use ClassType instead
                eqnType = IsEquationProgID(shp.OLEFormat.ClassType)
            End If
            If eqnType <> 0 Then
                If dir = "forward" Then
                    If shp.Range.start >= pos Then
                        If shp.Range.start - pos < closest Then
                            closest = shp.Range.start - pos
                            eqnIdx = i
                        End If
                    End If
                Else
                    If shp.Range.end <= pos Then
                        If pos - shp.Range.end < closest Then
                            closest = pos - shp.Range.end
                            eqnIdx = i
                        End If
                    End If
                End If
            End If
        Next
    End If

    If eqnIdx <> -1 Then
        Set shp = myRng.InlineShapes(eqnIdx)
        Set FindNextMTEqn = shp.Range
    Else
        Set FindNextMTEqn = Nothing
    End If
End Function

' returns the equation closer to the beginning of the doc if dir is "forward"
' or the equation closer to the end of the doc if dir is "backward"
Function closerEqn(eq1 As Range, eq2 As Range, dir As String)
    If eq1 Is Nothing Then
        Set closerEqn = eq2
    ElseIf eq2 Is Nothing Then
        Set closerEqn = eq1
    Else
        If dir = "forward" Then
            If eq1.start < eq2.start Then
                Set closerEqn = eq1
            Else
                Set closerEqn = eq2
            End If
        Else
            If eq1.end > eq2.end Then
                Set closerEqn = eq1
            Else
                Set closerEqn = eq2
            End If
        End If
    End If
End Function

Function InEqn(rng As Range, ByVal pos As Long) As Range
    Dim startPos As Long, endPos As Long
    Dim iEqn As Range, dEqn As Range
    Set InEqn = Nothing

    ' deal with a cursor position within a multichar delimiter
    Dim tmpRng As Range
    Set tmpRng = CopyRange(rng)
    tmpRng.SetRange pos - 1, pos + 1
    If tmpRng.Text = Strings.ChrW(&H5C) + "[" Then
        pos = pos + 1
    ElseIf tmpRng.Text = Strings.ChrW(&H5C) + "]" Then
        pos = pos - 1
    End If

    Set iEqn = FindTeXEqnFromWithin(rng, pos, "$", "$")
    Set dEqn = FindTeXEqnFromWithin(rng, pos, Strings.ChrW(&H5C) + "[", Strings.ChrW(&H5C) + "]")

    Dim inInline As Boolean, inDisplay As Boolean
    If Not iEqn Is Nothing Then
        inInline = IsValidTeX(iEqn)
    End If
    If Not dEqn Is Nothing Then
        inDisplay = IsValidTeX(dEqn)
    End If

    If inInline And inDisplay Then
        MsgBox "error: overlapping equations"
        Set InEqn = Nothing
    ElseIf inInline Then
        Set InEqn = iEqn
    ElseIf inDisplay Then
        Set InEqn = dEqn
    End If
End Function

Private Function FindTeXEqnFromWithin(rng As Range, pos As Long, _
    sDelim As String, eDelim As String) As Range
    Dim startPos As Long, endPos As Long
    Set FindTeXEqnFromWithin = CopyRange(rng)

    startPos = search(rng, "backward", pos, sDelim)
    If startPos = -1 Then
        Set FindTeXEqnFromWithin = Nothing
    Else
        endPos = search(rng, "forward", pos, eDelim)
        If endPos = -1 Then
            Set FindTeXEqnFromWithin = Nothing
        Else
            FindTeXEqnFromWithin.SetRange startPos, endPos + Len(eDelim)
        End If
    End If
End Function

Function IsAlphaNumeric(charStr As String)
    If (Asc(Strings.left$(charStr, 1)) >= 65 And Asc(Strings.left$(charStr, 1)) <= 90) Or _
        (Asc(Strings.left$(charStr, 1)) >= 97 And Asc(Strings.left$(charStr, 1)) <= 122) Or _
        (Asc(Strings.left$(charStr, 1)) >= 48 And Asc(Strings.left$(charStr, 1)) <= 57) Then
        IsAlphaNumeric = True
    Else
        IsAlphaNumeric = False
    End If
End Function

'returns true if it finds a single occurrence of chFind in strSearch
'this returns true:    FindSingleChar("abcd-efgh", "-")
'this returns false:   FindSingleChar("abcd--efgh", "-")
Function FindSingleChar(strSearch As String, chFind As String) As Boolean

    FindSingleChar = False

    Dim iPos As Long
    iPos = 1
   
    While (iPos >= 1)
        iPos = InStr(iPos, strSearch, chFind)
        If (iPos > 0) Then
            Dim iNextPos As Long
            iNextPos = iPos + 1
            Dim strFind As String
            strFind = chFind
            While (Strings.Mid(strSearch, iNextPos, 1) = chFind)
                strFind = strFind + chFind
                iNextPos = iNextPos + 1
            Wend
            If (Len(strFind) = 1) Then
                FindSingleChar = True
                Exit Function
            End If
            iPos = iPos + Len(strFind)
        End If
    Wend
    
End Function

Function IsValidTeX(rng As Range) As Boolean
    Dim startDelim As String, endDelim As String
    Dim texCode As String

    If rng.end - rng.start < 2 Then
        GoTo notTeX
    End If

    startDelim = Strings.left(rng.Text, 1)
    If startDelim = "$" Then
        endDelim = Strings.right(rng.Text, 1)
        If endDelim <> "$" Then
            GoTo notTeX
        End If
        texCode = Strings.left(rng.Text, Len(rng.Text) - 1)
        texCode = Strings.right(texCode, Len(texCode) - 1)
    Else
        If rng.end - rng.start < 4 Then
            GoTo notTeX
        End If
        startDelim = Strings.left(rng.Text, 2)
        If startDelim = Strings.ChrW(&H5C) + "[" Then
            endDelim = Strings.right(rng.Text, 2)
            If endDelim <> Strings.ChrW(&H5C) + "]" Then
                GoTo notTeX
            End If
            texCode = Strings.left(rng.Text, Len(rng.Text) - 2)
            texCode = Strings.right(texCode, Len(texCode) - 2)
        Else
            GoTo notTeX
        End If
    End If

    ' check for SOH characters (ASCII 1) which typically indicates the presence
    ' of an embedded OLE object such as a MT equation
    If InStr(texCode, Strings.Chr(1)) > 0 Then GoTo notTeX

    If Len(texCode) = 1 Then
        If Not IsAlphaNumeric(texCode) Then GoTo notTeX
    Else

        Dim specials As Boolean, otherDelims As Boolean
        specials = False
        otherDelims = False

        ' check for special characters
        If (InStr(texCode, "^") > 0) Or _
        (InStr(texCode, "_") > 0) Or _
        (InStr(texCode, "+") > 0) Or _
        (FindSingleChar(texCode, "-") = True) Or _
        (InStr(texCode, "*") > 0) Or _
        (InStr(texCode, "/") > 0) Or _
        (InStr(texCode, "=") > 0) Or _
        (InStr(texCode, "(") > 0) Or _
        (InStr(texCode, ")") > 0) Or _
        (InStr(texCode, "<") > 0) Or _
        (InStr(texCode, ">") > 0) Or _
        (InStr(texCode, "{") > 0) Or _
        (InStr(texCode, "}") > 0) Then
            specials = True
        End If

        ' only count a \ that isn't part of \$
        ' not the test here is only a crude heuristic, checking only the first \
        ' This is the lesser of two evils.  See MT-1984
        If (InStr(texCode, Strings.ChrW(&H5C)) > 0) Then
            If InStr(texCode, Strings.ChrW(&H5C)) <> InStr(texCode, Strings.ChrW(&H5C) + "$") Then
                specials = True
            End If
        End If

        ' check for embedded delimiters
        If (InStr(texCode, Strings.ChrW(&H5C) + "[") > 0) Or (InStr(texCode, Strings.ChrW(&H5C) + "]") > 0) Then
            otherDelims = True
        End If
        Dim sp As Integer
        sp = InStr(texCode, "$")
        If (sp > 1) Then
            If InStr(sp - 1, texCode, Strings.ChrW(&H5C)) < 1 Then
                otherDelims = True
            End If
        End If

        ' check for an initial long run of text
        If Len(texCode) > 40 Then
            Dim i As Integer
            Dim textRun As Boolean
            textRun = True
            For i = 1 To 40
                If Not IsAlphaNumeric(Strings.Mid(texCode, i, 1)) Then
                    textRun = False
                End If
            Next
            If textRun = True Then GoTo notTeX
        End If

        If Not specials Or otherDelims Or Len(texCode) > 500 Then
            GoTo notTeX
        End If
    End If

    IsValidTeX = True
    Exit Function

notTeX:
    IsValidTeX = False

End Function

' Search within rng, in direction dir, starting from pos, for query
' Disregard instanced of query that are TeX-escaped, i.e. \query
Public Function search(rng As Range, dir As String, pos As Long, query As String)
    Dim srchRng As Range
    Set srchRng = CopyRange(rng)

    Dim findDone As Boolean
    Dim thisPos As Long, lastPos As Long, cutoff As Integer
    findDone = False
    thisPos = -1
    lastPos = -1
    cutoff = 0

    'always search forward until lastPos <= queryPos <= thisPos
    Dim escapedMatch
    srchRng.find.ClearFormatting
    With srchRng.find
        .ClearAllFuzzyOptions
        .MatchWholeWord = False
        .MatchCase = False
        .forward = True
        .Wrap = wdFindStop
        .Text = query
        Do
            .Execute
            If .found Then
                ' only process it as a match if it is unescaped
                escapedMatch = False
                If (srchRng.start > 0) Then
                    Dim tmpRng As Range
                    Set tmpRng = CopyRange(srchRng)
                    tmpRng.SetRange tmpRng.start - 1, tmpRng.end
                    If tmpRng.Characters(1) = Strings.ChrW(&H5C) Then
                        escapedMatch = True
                    End If
                End If
                If Not escapedMatch Then
                   lastPos = thisPos
                   thisPos = srchRng.start
                   If thisPos >= pos Then
                        ' rng.find.Execute returns matches beyond the end of
                        ' rng. The docs aren't clear enough to call this a bug,
                        ' but, we have to work around this special feature.
                        ' MT-2102
                        If thisPos > rng.end Then
                           thisPos = -1
                        End If
                        findDone = True
                   End If
                End If
            Else
                findDone = True
            End If
            cutoff = cutoff + 1
            If cutoff > loopCutoff Then findDone = True
        Loop Until findDone
    End With

    search = -1
    If thisPos <> -1 Then
        If dir = "forward" Then
            If thisPos >= pos Then
                search = thisPos
            End If
        Else
            If thisPos >= pos Then
                search = lastPos
    Else
                search = thisPos
            End If
        End If
    End If
End Function

Function IsLikelyDisplayEqn(rng As Range) As Boolean
    Dim tmpRng As Range

    IsLikelyDisplayEqn = False

    'If we are in a story that doesn't support display equations, we may not
    'be able to obtain the document width, and thus the call to
    'CreateDisplayEquationsStyle will blow up.  This happens in headers, for
    'example, MT-2101. Consequently, we don't even consider promoting likely
    'display equations in places where display equations aren't permitted
    If Not isDisplayEqnPermitted(rng) Then Exit Function

    Set tmpRng = CopyRange(rng)
    tmpRng.SetRange rng.start - 1, rng.end + 1
    If (Strings.left(tmpRng.Text, 1) = Strings.Chr(13) Or Strings.left(tmpRng.Text, 1) = Strings.Chr(9)) Or tmpRng.start = 0 And _
        Strings.right(tmpRng.Text, 1) = Strings.Chr(13) Then
        CreateDisplayEquationStyle ActiveDocument
        If rng.style = ActiveDocument.Styles(mtstyle_DISPLAY_EQUATION) Then
            IsLikelyDisplayEqn = True
        Else
            ' centering is stored on a per character basis
            Dim myChars As Characters
            Set myChars = rng.Characters
            If myChars.count > 0 Then
                If myChars.First.ParagraphFormat.Alignment = wdAlignParagraphCenter Then
                    IsLikelyDisplayEqn = True
                End If
            End If
        End If
    End If

End Function

Function isDisplayEqnPermitted(rng As Range) As Boolean
    ' TODO
    ' There is a problem in that the central state checking code is in the UI.
    ' This code just duplicates the functionality for MTW6.5 and should be replaced in MT7
    isDisplayEqnPermitted = False

    'check to see if a document is open
    If Documents.count = 0 Then Exit Function

    'check to see if our cursor is in an OK location
    If (Selection.Information(wdInFootnoteEndnotePane) Or _
        Selection.Information(wdInCommentPane) Or _
        Selection.Information(wdInHeaderFooter)) Then Exit Function

    If Val(Application.version) = kWordX Then
        If (ActiveDocument.ActiveWindow.View.SplitSpecial = MTPaneRevisions And _
            ActiveDocument.ActiveWindow.ActivePane.index > 1) Then Exit Function
    End If

    'check if this command is allowed in the current view
    If (Val(Application.version) = kWord2003 And ActiveWindow.View.Type = MTReadingView) Then Exit Function

    'make sure we're not in an EB eqn
    If (Val(Application.version) >= kWord2007 And SelInEBEquation) Then Exit Function

    'make sure we're not in a text box
    If (Selection.storyType = wdTextFrameStory) Then Exit Function

    isDisplayEqnPermitted = True
 End Function

Function isDisplayTeXEqn(rng As Range) As Boolean
    isDisplayTeXEqn = (Strings.left(rng.Text, 2) = Strings.ChrW(&H5C) + "[") And (Strings.right(rng.Text, 2) = Strings.ChrW(&H5C) + "]")
End Function

Function isInlineTeXEqn(rng As Range) As Boolean
    isInlineTeXEqn = (Strings.left(rng.Text, 1) = "$") And (Strings.right(rng.Text, 1) = "$")
End Function

Function isMTEqn(rng As Range) As Boolean
    If rng.InlineShapes.count <> 1 Then
        isMTEqn = False
    Else
        Dim shp As InlineShape
        Dim eqnType As Integer
        Set shp = rng.InlineShapes(1)
        eqnType = 0
        If shp.Type = wdInlineShapeEmbeddedOLEObject Then
            'accessing ProgID may cause an error 5825 on corrupt objects, so use ClassType instead
            eqnType = IsEquationProgID(shp.OLEFormat.ClassType)
        End If
        isMTEqn = eqnType <> 0
    End If
End Function

Sub toggleEqn(eqnRng As Range, Optional ShowDialog As Boolean = True)
    If isInlineTeXEqn(eqnRng) Then
        toggleInlineTeXEqn eqnRng, ShowDialog
    ElseIf isDisplayTeXEqn(eqnRng) Then
        toggleDisplayTeXEqn eqnRng, ShowDialog
    ElseIf isMTEqn(eqnRng) Then
        toggleMTEqn eqnRng
    Else
        MTLib.WriteLog "can't determine equation type"
    End If
End Sub


' invalid location for eqn entirely is dealt with in the UI template in MTTeXToggle()
Sub toggleInlineTeXEqn(rng As Range, Optional ShowDialog As Boolean = True)
    Dim result As String
    If IsLikelyDisplayEqn(rng) And ShowDialog Then
        rng.Select 'to show the user which equation we are talking about
        result = showConvertToDisplayDlg()
        If result = "cancel" Then
            Exit Sub
        ElseIf result = "display" Then
            replaceTeXWithMT rng, "display"
            'reset alignment to left, if it was centered
            rng.ParagraphFormat.Alignment = wdAlignParagraphLeft
            Exit Sub
        End If
    End If
    replaceTeXWithMT rng, "inline"
End Sub

' invalid location for eqn entirely is dealt with in the UI template in MTTeXToggle()
Sub toggleDisplayTeXEqn(rng As Range, Optional ShowDialog As Boolean = True)
    Dim result As String
    If Not isDisplayEqnPermitted(rng) And ShowDialog Then
        rng.Select 'to show the user which equation we are talking about
        result = showConvertToInlineDlg()
        If result = "cancel" Then
            Exit Sub
        Else
            replaceTeXWithMT rng, "inline"
            Exit Sub
        End If
    End If
    replaceTeXWithMT rng, "display"
End Sub


Sub toggleMTEqn(rng As Range)
    replaceMTWithTeX rng
    If (Not isDisplayEqnPermitted(rng)) And isDisplayTeXEqn(rng) Then
       ForceInlineDelimsOnEqnRange rng
    End If
End Sub

Function showConvertToInlineDlg() As String
    gMTConvertToInlineResult = "cancel"
    MTConvertToInlineDlg.Show
    showConvertToInlineDlg = gMTConvertToInlineResult
End Function

Function showConvertToDisplayDlg() As String
    gMTConvertToDisplayResult = "cancel"
    MTConvertToDisplayDlg.Show
    showConvertToDisplayDlg = gMTConvertToDisplayResult
End Function

'converts valid display delims to inline delims
'does nothing if valid display delims are not found
Sub DisplayToInlineDelims(ByRef texStr As String)
    Dim startDelim As String, endDelim As String
    If Len(texStr) >= 4 Then
        startDelim = Strings.left(texStr, 2)
        If startDelim = Strings.ChrW(&H5C) + "[" Then
            endDelim = Strings.right(texStr, 2)
            If endDelim = Strings.ChrW(&H5C) + "]" Then
                texStr = Strings.left(texStr, Len(texStr) - 2)
                texStr = Strings.right(texStr, Len(texStr) - 2)
                texStr = "$" & texStr & "$"
            End If
        End If
    End If
End Sub


'converts valid inline delims to display delims
'does nothing if valid inline delims are not found
Sub InlineToDisplayDelims(ByRef texStr As String)
    Dim startDelim As String, endDelim As String
    If Len(texStr) >= 2 Then
        startDelim = Strings.left(texStr, 1)
        If startDelim = "$" Then
            endDelim = Strings.right(texStr, 1)
            If endDelim = "$" Then
                texStr = Strings.left(texStr, Len(texStr) - 1)
                texStr = Strings.right(texStr, Len(texStr) - 1)
                texStr = Strings.ChrW(&H5C) & "[" & texStr & Strings.ChrW(&H5C) & "]"
            End If
        End If
    End If
End Sub

Sub ForceInlineDelimsOnEqnRange(rng As Range)
    Dim texCode As String
    If rng.end - rng.start < 4 Then
        MTLib.WriteLog "Can't force inline delimiters for: " & rng.Text
        Exit Sub
    Else
        texCode = rng.Text
        DisplayToInlineDelims texCode
        rng.delete
        rng.InsertAfter texCode
    End If
End Sub

'Ensures that the position at the end of the eqnRng argument is properly prepared.
'In the generic case, that means with a paragraph and tab before it, and a paragraph after it
'special cases for tables, EB equations, beginning and end of document, etc are also handled.
'It does not apply the MTDisplayEquation style.
'It shares code with MTInsertEquation for preparing the insertion point for MT equations
'created via the UI.
Sub PrepareDisplayTogglePosition(eqnRng As Range)
    Dim savedSelection As Selection
        Dim alreadyOkay As Boolean
        Dim distMoved

    Set savedSelection = Selection
    eqnRng.Select

    ' this code and MTInsertEquation.PrepareInsertionPostion modify the Selection
    With Selection
        .Collapse wdCollapseEnd
        alreadyOkay = True

        distMoved = .moveLeft(wdCharacter, 1, wdMove)
        If distMoved = 0 Then 'if at start of document
            alreadyOkay = False
        Else
            If Strings.left$(.Text, 1) <> vbTab Then
                distMoved = .MoveRight(wdCharacter, 1, wdMove)
                alreadyOkay = False
            Else
                distMoved = .MoveRight(wdCharacter, 1, wdMove)
                If Strings.left$(.Text, 1) <> Strings.Chr(13) And Strings.left$(.Text, 1) <> "." And Strings.left$(.Text, 1) <> vbTab Then
                    alreadyOkay = False
                End If
            End If
        End If

        If alreadyOkay = False Then
            MTInsertEquation.PrepareInsertionPostion
            .TypeText vbTab
        End If
    End With

    Set eqnRng = Selection.Range ' return updated range by reference
    savedSelection.Select
End Sub

Sub replaceTeXWithMT(eqnRng As Range, mode As String)
    Dim texStr As String
    Dim startPos, endPos
    Dim mtEqn As InlineShape
    Dim repositionCursor As Boolean
    Dim positionBefore As Boolean

    On Error GoTo err
    MTLib.SetScreenUpdate False

    ' if the user selection is an insertion point in a TeX eqn,
    ' we want to leave the cursor at the appropriate end of the
    ' resulting MT eqn
    positionBefore = False
    repositionCursor = False
    If Selection.start = Selection.end And _
        (Selection.start >= eqnRng.start And Selection.end <= eqnRng.end) Then
        repositionCursor = True
        If Selection.start <= (eqnRng.end + eqnRng.start) / 2 Then
            positionBefore = True
        End If
    End If

    texStr = eqnRng.Text

    ' at the beginning of the document, deleting the eqution range
    ' deletes the bookmark indicating the start of the toggle range
    Dim haveToggleStart As Boolean, haveToggleEnd As Boolean
    haveToggleStart = ActiveDocument.Bookmarks.Exists("MTToggleStart")
    haveToggleEnd = ActiveDocument.Bookmarks.Exists("MTToggleEnd")

    eqnRng.delete

    ' When there is a newline in the TeX markup, deleting the range
    ' will insert a newline in its place in >W2007 (MT-2191), unless the
    ' expression is at the beginning of a story.  We have to correct for
    ' that in order to get the expected toggle behavior.  MT-1930
    If InStr(texStr, vbCr) > 0 And eqnRng.start > 0 Then
        Dim distMoved
        distMoved = eqnRng.MoveEnd(wdCharacter, 1)
        If distMoved > 0 And eqnRng.Text = vbCr Then
            eqnRng.delete
        Else 'undo moving the end point (MT-2191)
            eqnRng.Collapse wdCollapseStart
        End If
    End If

    If haveToggleStart And Not ActiveDocument.Bookmarks.Exists("MTToggleStart") Then
        ActiveDocument.Bookmarks.Add name:="MTToggleStart", Range:=eqnRng
    End If
    If haveToggleEnd And Not ActiveDocument.Bookmarks.Exists("MTToggleEnd") Then
        ActiveDocument.Bookmarks.Add name:="MTToggleEnd", Range:=eqnRng
    End If

    If mode = "display" Then
        PrepareDisplayTogglePosition eqnRng
    End If

    Dim blankEqnPath As String
    blankEqnPath = MTLib.GetMathTypeDir & Application.PathSeparator & "Office Support" & Application.PathSeparator & "BlankEqn.doc"

    eqnRng.InsertFile blankEqnPath, "MTBlankEqn"
    'Delete the MTBlankEqn bookmark which comes in with the equation
    If ActiveDocument.Bookmarks.Exists("MTBlankEqn") Then
        ActiveDocument.Bookmarks("MTBlankEqn").delete
    End If
    MTLib.WriteLog "Blank MT Eqn inserted"

    ' Word 2007 appears not to expand the rng to include the new blank equation
    ' while Word 2003 does.  Apparently in Word 2007 we have to expand the range
    ' by the width of our field code text "{ EMBED Equation.DSMT4 }" which is
    ' 26 chars long, since the braces are actually two-byte control sequences.
    ' Hmm.  Surely, there is a better way...
    eqnRng.SetRange eqnRng.start, eqnRng.end + 27

    If eqnRng.InlineShapes.count > 0 Then
        Set mtEqn = eqnRng.InlineShapes(1)
    Else
        Dim msg As String
        msg = MTLib.GetUserString2("1675", "3275", "TeX Toggle Failed: couldn't locate blank equation")
        MsgBox msg, vbExclamation
        Exit Sub
    End If

    If mode = "display" Then
        Set eqnRng = mtEqn.Range
        MTInsertEquation.ApplyDisplayStyle eqnRng
    End If

    ' set the new equation from the Texvc code
    If mode = "display" Then
        InlineToDisplayDelims texStr
                Else
        DisplayToInlineDelims texStr
                End If
    SetMTTeXData mtEqn, texStr
    Set eqnRng = mtEqn.Range

    'finally put the cursor at the correct end of the equation, if appropriate
    If repositionCursor Then
        Dim rng As Range
        Set rng = mtEqn.Range
        If positionBefore Then
            rng.Collapse wdCollapseStart
        Else
            rng.Collapse wdCollapseEnd
        End If
        ActiveDocument.Bookmarks("MTToggleStart").start = rng.start
        ActiveDocument.Bookmarks("MTToggleStart").end = rng.end
        ActiveDocument.Bookmarks("MTToggleEnd").start = rng.start
        ActiveDocument.Bookmarks("MTToggleEnd").end = rng.end
    End If

    MTLib.SetScreenUpdate True
    Exit Sub

err:
    msg = MTLib.GetUserString2("1670", "3270", "Error ") & err.Number & MTLib.GetUserString2("1671", "3271", " occurred, ") & err.Description
    MsgBox msg, vbExclamation
    MTLib.SetScreenUpdate True
End Sub

Sub replaceMTWithTeX(eqnRng As Range)
    Dim positionAfter As Boolean
    Dim eqnShape As InlineShape
    Set eqnShape = eqnRng.InlineShapes(1)

    ' if the Selection is immediately after the equation, we have
    ' to take steps to ensure it remains there after toggling
    positionAfter = False
    If ActiveDocument.Bookmarks.Exists("MTToggleStart") And _
       ActiveDocument.Bookmarks.Exists("MTToggleStart") Then
        Dim ss, se, es, ee
        ss = ActiveDocument.Bookmarks("MTToggleStart").start
        se = ActiveDocument.Bookmarks("MTToggleStart").end
        es = ActiveDocument.Bookmarks("MTToggleEnd").start
        ee = ActiveDocument.Bookmarks("MTToggleEnd").end
        If ss = eqnRng.end And se = eqnRng.end And es = eqnRng.end And ee = eqnRng.end Then
            positionAfter = True
        End If
    End If

    Dim texStr As String
    texStr = GetMTTeXData(eqnShape)

    ' delete the old MT OLE equation and replace with TeX
    Dim rng As Range
    Set rng = eqnShape.Range

    ' at the beginning of the document, deleting the equation range
    ' deletes the bookmark indicating the start of the toggle range
    Dim haveToggleStart As Boolean
    haveToggleStart = ActiveDocument.Bookmarks.Exists("MTToggleStart")

    ' A text box containing only an MT equation is deleted whenever the equation
    ' is deleted. But if there is text along with the MT equation, deleting
    ' the text and the MT equation doesn't remove the text box (even though
    ' the deletion leave the text box empty!) So put placeholder text beside
    ' the equation.
    rng.InsertBefore "x"
    rng.delete
    If haveToggleStart And Not ActiveDocument.Bookmarks.Exists("MTToggleStart") Then
        ActiveDocument.Bookmarks.Add name:="MTToggleStart", Range:=eqnRng
    End If

    rng.InsertBefore texStr

    If positionAfter Then
        ActiveDocument.Bookmarks("MTToggleStart").start = rng.end
        ActiveDocument.Bookmarks("MTToggleStart").end = rng.end
        ActiveDocument.Bookmarks("MTToggleEnd").start = rng.end
        ActiveDocument.Bookmarks("MTToggleEnd").end = rng.end
    End If

    Set eqnRng = rng
End Sub

Sub SetMTTeXData(eqnShape As InlineShape, texStr As String)

    On Error GoTo shutdown

    Dim myObj As Object
    Dim stat As Long

    MTLib.WriteLog "Entering SetMTTeXData"

    ActivateMT eqnShape
    Set myObj = eqnShape.OLEFormat.Object
    
    Dim mmlUnicode() As Byte
    mmlUnicode = texStr
    stat = MTSetEqnFromLangStr(myObj, mtlangTEX_INPUT, mmlUnicode(0), Len(texStr))

    MTLib.WriteLog "MTSetEqnFromLangStr called successfully"

shutdown:
    MTLib.WriteLog "Shutting down in SetMTTeXData"
    stat = ShutdownMT(eqnShape)
    MTLib.WriteLog "Exiting SetMTTeXData"

End Sub

Function GetMTTeXData(eqnShape As InlineShape)
    On Error GoTo shutdown

    Dim myObj As Object
    Dim texStr As String
    Dim stat As Long
    Dim texlen As Long

    ActivateMT eqnShape
    Set myObj = eqnShape.OLEFormat.Object

    stat = MTGetLangStrFromEqn(myObj, mtlangTEX_INPUT, vbNullString, texlen)
    If (stat = mtOK And texlen > 0) Then
       texStr = Strings.Space(texlen)
       stat = MTGetLangStrFromEqn(myObj, mtlangTEX_INPUT, texStr, texlen)
       texStr = RemoveNull(texStr)
       MTLib.WriteLog "MTGetLangStrFromEqn called successfully"
    End If

shutdown:
    MTLib.WriteLog "Shutting down in GetMTTeXData"
    stat = ShutdownMT(eqnShape)
    MTLib.WriteLog "Exiting GetMTTeXData"

    GetMTTeXData = texStr

End Function

Public Function CopyRange(ByVal rng As Range) As Range
    Dim saved As Range
    MTLib.SetScreenUpdate False
    Set saved = Selection.Range
    rng.Select
    Set CopyRange = Selection.Range ' makes a copy, not a reference
    If Strings.right(CopyRange.Text, 2) = Strings.Chr(13) & Strings.Chr(7) Then CopyRange.SetRange CopyRange.start, CopyRange.end - 1
    saved.Select
    MTLib.SetScreenUpdate True
End Function
