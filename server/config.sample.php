<?php
// Copy as config.php to change where MyRoots keeps its data.
// Best: a folder OUTSIDE the public web folder, e.g. one level up.
// Move the existing data folder there before you save this file.
define('MR_DATA', dirname(__DIR__) . '/myroots-data');
