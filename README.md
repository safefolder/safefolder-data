# What is Safefolder?

Our mission is to provide the tools for users to store all their data on their devices and control who has access to it.

This repository contains the under-construction data layer for the Safefolder platform. It will allow the separation of user data on local devices and nonuser data in a cloud, enhancing users' security and privacy on the net.

You can watch this YouTube explainer video:

<a href="https://www.youtube.com/watch?v=IRfGsF6DrkU" target="_blank"><img src="https://img.youtube.com/vi/IRfGsF6DrkU/0.jpg" alt="Explainer Video"></a>

The technology preview video with the user interface and experience:

<a href="https://www.youtube.com/watch?v=uK1ARuXs_ME" target="_blank"><img src="https://img.youtube.com/vi/uK1ARuXs_ME/0.jpg" alt="Tech Preview Video"></a>

You can also check our site at:

https://safefolder.app

Safefolder has a digital property backed by an NFT, which gives access to unlimited usage in the future, a 3x1 reward for the current stage, Bootstrapping. We call these “Spots.” A spot will allow you to use Safefolder’s repositories in your cloud. And gives you access to a 70% shared revenue affiliate network.

# Status

This repository is currently under construction. We are adapting it to align with the user experience presented in the video above and adding unit tests. The functionality for updating tables (folders) still needs to be completed. Below, you can check which data statements have been implemented.

The documentation related to requirements and functional analysis is under construction. The changes are not big, but some refactoring is needed to adapt to data channels and workspaces. If you want to help us out, you can check the section about contributing.

# Folder Data Structure

User data needs are pretty broad. In software, we use databases with tables, although users, we believe, “feel” about data from a broader perspective. They sense folders as containers of files and data, which is why this project's folder is the data entity.

It allows us to have a universal storage structure of files, documents, tabular data, and any data we can think of. We also have the functionality of subfolders, so users intuitively organize data.

# Contributing

We have a collaborative economy. We handle contributors from our web page. Right now, we look forward to contributors who can commit regularly. Details are on our web page.

You can create an issue if you want to provide feedback on a bug.

If you are planning for some new feature from the repository, the first step is to create an issue with the label “feature request.” This way, we know your planned enhancement. This allows you not to commit regularly but contribute in an atomic way.

Contributing regularly, at least for a quarter, gives you access to own a Spot. This allows you to use our software self-managed, hosting it in your systems. You can also use it in our cloud and access our affiliate network, which offers 70% rewards and 3x1 benefits.

# Fields

We have 29 fields that cover pretty much all the needs of storing data.

1.	**Name**: All folders have an ID, but they also have a human way of referencing the folder item. This allows for different field types for this name.
2.	**SmallText**: Text field.
3.	**LongText**: Description type long texts.
4.	**CheckBox**
5.	**Select**: Selection box with different options. Allows many.
6.	**Date**: The date and time fields have different date formats.
7.	**Duration**: Duration of a phone call, for example.
8.	**Number**: Number with formatting support.
9.	AuditTime: Date and time an item was created or modified.
10.	**AuditBy**: Who created or modified an item?
11.	**Currency**: Prices.
12.	**Percent**: Percentages.
13.	**Formula**: Like in any spreadsheet application, a formula handles different functions and allows a dynamic field with data from other fields.
14.	**Link**: Link to another folder. We store the IDs in both folders. Additional fields needed to display data can be integrated into the Reference field. One or many relationships.
15.	**Reference**: When an item is saved, we include the references defined in the folder structure. This allows you to display and query that data.
16.	**Language**: We support full-text queries and indexing. This field allows us to define the language, so full text supports that. We also use automatic detection of language from the fields defined like Text.
17.	**Text**: Text to be indexed in full-text mode. It can be a copy of other fields or content you may want to include.
18.	**GenerateId**: Generate an id.
19.	**GenerateNumber**: Generate a new number in a sequence.
20.	**Phone**: Phone field.
21.	**Email**: Email field.
22.	**Url**: Url field.
23.	**Rating**: Rating like a 5-star system.
24.	**Set**: A list of items when ordering is not needed.
25.	**Object**: An object like a map of key->value.
26.	**Subfolder**: Links to another item in a subfolder.
27.	**Stats**: Offers statistics with minimum, maximum, average, etc.
28.	**File**: File that supports checking content, grabbing metadata, and analyzing language.
29.	**Statement**: Execute a statement and place output into the field.

# Functions

Integrated into the formula field. Formula fields allow you to use functions. Many of them are the ones you know in LibreOffice or Excel.

1.	CONCAT
2.	FORMAT
3.	JOINLIST
4.	LENGTH
5.	LOWER
6.	UPPER
7.	REPLACE
8.	DATE
9.	DATEFMT
10.	DAY
11.	DAYS
12.	HOUR
13.	MONTH
14.	NOW
15.	SECOND
16.	MINUTE
17.	TODAY
18.	WEEK
19.	WEEKDAY
20.	YEAR
21.	IF
22.	MID
23.	REPT
24.	SUBSTITUTE
25.	TRIM
26.	CEILING
27.	COUNT
28.	COUNTA
29.	COUNTALL
30.	EVEN
31.	EXP
32.	FLOOR
33.	INT
34.	LOG
35.	MAX
36.	MIN
37.	MOD
38.	POWER
39.	ROUND
40.	ROUNDDOWN
41.	ROUNDUP
42.	SQRT
43.	VALUE
44.	CREATED_TIME
45.	DATEADD
46.	DATEDIFF
47.	LAST_MODIFIED_TIME
48.	RECORD_ID
49.	TRUE
50.	FALSE

# Statements

Statements follow a SQL-like syntax. We support these:

1. CREATE FOLDER
2. INSERT INTO FOLDER
3. SELECT
4. SELECT COUNT(*)
5. GET
6. SELECT GROUP BY
7. DESCRIBE
8. DROP FOLDER
9. ADD COLUMN
10. MODIFY COLUMN
11. LIST FOLDERS

# Examples

The output is an YAML document.

    ./safefolder-data run statement --statement '
    CREATE FOLDER Account (
        NAME COLUMN SmallText,
        "Email" SmallText WITH Required=true,
        "Password Fingerprint" SmallText WITH Required=true,
        "Created By" CreatedBy,
        "Created On" CreatedTime,
        "Last Modified By" LastModifiedBy,
        "Last Modified On" LastModifiedTime,
    );
    '

    ./safefolder-data run statement --statement '
    INSERT INTO FOLDER Site (
        Name = Kutch Inc,
    ),
    (
        Name = McClure - Parker,
    ),
    (
        Name = Kling LLC,
    ),
    (
        Name = Wehner - Amore,
    ),
    (
        Name = Metz and Farrel,
    ),
    (
        Name = Gerhold and Rice,
    ),
    (
        Name = Wisozk and MacGyver,
    );
    '

    ./safefolder-data run statement --statement '
    ADD COLUMN INTO MyTasks (
        "Category" Select WITH Options={Management|Errands|Personal},
    );
    '

    ./safefolder-data run statement --statement '
    SELECT * FROM "MyTasks";
    '
